use std::time::Duration;

use rand::Rng;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use tracing::{field::display, Span};
use uuid::Uuid;

use crate::{
    configuration::{RetrySettings, Settings},
    domain::SubscriberEmail,
    email_client::EmailClient,
    startup::get_connection_pool,
};

#[derive(Debug)]
pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

#[tracing::instrument(skip_all)]
pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
    retry: &RetrySettings,
) -> Result<ExecutionOutcome, anyhow::Error> {
    let task = dequeue_task(pool).await?;

    if task.is_none() {
        return Ok(ExecutionOutcome::EmptyQueue);
    }

    let (transaction, issue_id, email, mut n_retries) = task.unwrap();

    Span::current()
        .record("newsletter_issue_id", display(issue_id))
        .record("subscriber_email", display(&email));

    match SubscriberEmail::parse(email.clone()) {
        Ok(email) => {
            let issue = get_issue(pool, &issue_id).await?;
            match email_client
                .send_email(
                    &email,
                    &issue.title,
                    &issue.html_content,
                    &issue.text_content,
                )
                .await
            {
                Ok(_) => {
                    delete_task(transaction, &issue_id, email.as_ref()).await?;
                    Ok(ExecutionOutcome::TaskCompleted)
                }
                Err(e) => {
                    if n_retries < retry.max_retries {
                        tracing::error!(
                            error.cause_chain = ?e,
                            error.message = %e,
                            "Failed to deliver to a confirmed subscriver. Retrying."
                        );
                        n_retries += 1;
                        if let Err(e) = reschedule_task(
                            transaction,
                            &issue_id,
                            &email,
                            n_retries,
                            exponential_backoff_equal_jitter(
                                retry.cap_sec,
                                retry.base_sec,
                                n_retries,
                            ),
                        )
                        .await
                        {
                            tracing::error!(
                                error.cause_chain = ?e,
                                error.message = %e,
                                "Failed to reschedule task. Rolling back transaction."
                            );
                            return Err(anyhow::anyhow!("Failed to reschedule task: {}", e));
                        };
                        Ok(ExecutionOutcome::TaskCompleted)
                    } else {
                        delete_task(transaction, &issue_id, email.as_ref()).await?;
                        Err(anyhow::anyhow!(
                            "Max retries reached.\nNewsletter: '{}'\n User: {}",
                            issue_id,
                            email.as_ref()
                        ))
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Skipping a confirmed subscriber. Their stored contact details are invalid."
            );
            delete_task(transaction, &issue_id, &email).await?;
            Ok(ExecutionOutcome::TaskCompleted)
        }
    }
}

type PgTransaction = Transaction<'static, Postgres>;

#[tracing::instrument(name = "Dequeue task", skip_all)]
async fn dequeue_task(
    pool: &PgPool,
) -> Result<Option<(PgTransaction, Uuid, String, u16)>, anyhow::Error> {
    let mut transaction = pool.begin().await?;

    let row = sqlx::query!(
        r#"
        SELECT newsletter_issue_id, subscriber_email, n_retries
        FROM issue_delivery_queue
        WHERE
            execute_after < now() OR execute_after IS NULL
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
        "#
    )
    .fetch_optional(&mut *transaction)
    .await?;

    if let Some(row) = row {
        Ok(Some((
            transaction,
            row.newsletter_issue_id,
            row.subscriber_email,
            row.n_retries.try_into().unwrap(),
        )))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(name = "Reschedule task", skip_all)]
async fn reschedule_task(
    mut transaction: PgTransaction,
    issue_id: &Uuid,
    email: &SubscriberEmail,
    n_retries: u16,
    delay_seconds: f64,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        UPDATE issue_delivery_queue
        SET
            n_retries = $3,
            execute_after = now() + ((interval '1 sec') * $4)
        WHERE
            newsletter_issue_id = $1 AND
            subscriber_email = $2
        "#,
        issue_id,
        email.as_ref(),
        n_retries as i32,
        delay_seconds
    );
    transaction.execute(query).await?;
    Ok(())
}

/// Using: https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/
fn exponential_backoff_equal_jitter(cap: u64, base: u64, attempt: u16) -> f64 {
    let max_delay = cap.min(base * 2u64.pow(attempt.into()));
    let temp = max_delay / 2;
    (temp + rand::thread_rng().gen_range(0..=temp)) as f64
}

#[tracing::instrument(name = "Delete task", skip_all)]
async fn delete_task(
    mut transaction: PgTransaction,
    issue_id: &Uuid,
    email: &str,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM issue_delivery_queue
        WHERE newsletter_issue_id = $1 AND
            subscriber_email = $2"#,
        issue_id,
        email
    );
    transaction.execute(query).await?;
    transaction.commit().await?;
    Ok(())
}

#[derive(Debug)]
struct NewsletterIssue {
    title: String,
    text_content: String,
    html_content: String,
}

#[tracing::instrument(name = "Get Newsletter Issue", skip_all)]
async fn get_issue(pool: &PgPool, issue_id: &Uuid) -> Result<NewsletterIssue, anyhow::Error> {
    let issue = sqlx::query_as!(
        NewsletterIssue,
        r#"
        SELECT title, text_content, html_content
        FROM newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(pool)
    .await?;
    Ok(issue)
}

async fn worker_loop(
    pool: PgPool,
    email_client: EmailClient,
    retry: RetrySettings,
) -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task(&pool, &email_client, &retry).await {
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Ok(ExecutionOutcome::TaskCompleted) => {}
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);
    let retry = configuration.email_client.clone().retry;
    let email_client = configuration.email_client.client();
    worker_loop(connection_pool, email_client, retry).await
}
