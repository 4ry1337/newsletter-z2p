use std::fmt::{Debug, Display};

use newsletter::{
    configuration::get_configuration,
    expiry_requrests, issue_delivery_worker,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("newsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration.clone()).await?;
    let appication_task = tokio::spawn(application.run_until_stopped());
    let delivery_task = tokio::spawn(issue_delivery_worker::run_worker_until_stopped(
        configuration.clone(),
    ));
    let requrest_task = tokio::spawn(expiry_requrests::run_worker_until_stopped(configuration));

    tokio::select! {
        outcome = appication_task => report_exit("API", outcome),
        outcome = delivery_task => report_exit("Background worket", outcome),
        outcome = requrest_task => report_exit("Background worket", outcome),
    };
    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has existed", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed to complete",
                task_name
            )
        }
    }
}
