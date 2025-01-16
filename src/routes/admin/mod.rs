mod dashboard;
mod logout;
mod newsletter;
mod password;

pub use dashboard::admin_dashboard;
pub use logout::log_out;
pub use newsletter::*;
pub use password::*;

use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(name = "Get username by user_id", skip(pool))]
pub async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(r#"SELECT username FROM users WHERE user_id = $1"#, user_id)
        .fetch_one(pool)
        .await
        .context("Faield to perform a  query to retrive a username")?;
    Ok(row.username)
}
