use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::helpers::spawn_app;

#[sqlx::test]
async fn health_check_works(_: PgPoolOptions, options: PgConnectOptions) {
    let pool = PgPoolOptions::new().connect_with(options).await.unwrap();
    let app = spawn_app(pool).await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
