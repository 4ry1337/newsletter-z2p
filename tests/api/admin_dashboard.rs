use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[sqlx::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard(
    _: PgPoolOptions,
    options: PgConnectOptions,
) {
    let pool = PgPoolOptions::new().connect_with(options).await.unwrap();
    let app = spawn_app(pool).await;

    let response = app.get_admin_dashboard().await;

    assert_is_redirect_to(&response, "/login");
}

#[sqlx::test]
async fn logout_clears_session_state(_: PgPoolOptions, options: PgConnectOptions) {
    let pool = PgPoolOptions::new().connect_with(options).await.unwrap();
    let app = spawn_app(pool).await;

    let response = app
        .post_login(&serde_json::json!({
            "username": &app.test_user.username,
            "password": &app.test_user.password,
        }))
        .await;

    assert_is_redirect_to(&response, "/admin/dashboard");

    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("<p>Welcome, {}!</p>", app.test_user.username)));

    let response = app.post_logout().await;

    assert_is_redirect_to(&response, "/login");

    let html_page = app.get_login_html().await;
    assert!(html_page.contains("<p><i>You have successfully logged out.</i></p>"));

    let response = app.get_admin_dashboard().await;

    assert_is_redirect_to(&response, "/login");
}
