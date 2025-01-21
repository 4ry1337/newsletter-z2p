use std::time::Duration;

use sqlx::{postgres::types::PgInterval, PgPool};

use crate::{
    configuration::{IdempotencySettings, Settings},
    startup::get_connection_pool,
};

#[tracing::instrument(skip_all)]
pub async fn try_execute_task(pool: &PgPool, expire_in_sec: u64) -> Result<(), anyhow::Error> {
    let expiration_interval: PgInterval = Duration::from_secs(expire_in_sec).try_into().unwrap();
    match sqlx::query!(
        r#"
        DELETE FROM idempotency
        WHERE (created_at + $1) < now()
        "#,
        expiration_interval
    )
    .execute(pool)
    .await
    {
        Ok(query) => {
            tracing::info!(
                "Removed {} expired idempotency keys.",
                query.rows_affected()
            );
            Ok(())
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Unable to clear expired idempotency keys."
            );
            Ok(())
        }
    }
}

async fn worker_loop(pool: PgPool, idempotency: IdempotencySettings) -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task(&pool, idempotency.expire_in_sec).await {
            Ok(_) => {
                tokio::time::sleep(Duration::from_secs(idempotency.check_sec)).await;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);
    worker_loop(connection_pool, configuration.application.idempotency).await
}
