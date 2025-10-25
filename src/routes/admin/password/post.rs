use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    authentication::{self, validate_credentials, AuthError, Credentials, UserId},
    routes::get_username,
    utils::{e500, see_other}
};

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    current_password:   SecretString,
    new_password:       SecretString,
    new_password_check: SecretString
}

#[tracing::instrument(
    name="Change password",
    skip(form, pool, user_id),
    fields(user_id=%*user_id)
)]
pub async fn change_password(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match."
        )
        .send();
        return Ok(see_other("/admin/password"));
    }

    if form.new_password.expose_secret().graphemes(true).count() > 129 {
        FlashMessage::error("Password must be at most 129 characters.").send();
        return Ok(see_other("/admin/password"));
    }

    if form.new_password.expose_secret().graphemes(true).count() < 12 {
        FlashMessage::error("Password must be at least 12 characters.").send();
        return Ok(see_other("/admin/password"));
    }

    let username = get_username(*user_id, &pool).await.map_err(e500)?;

    let credentials = Credentials {
        username,
        password: form.0.current_password
    };

    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            },
            AuthError::UnexpectedError(_) => Err(e500(e))
        };
    }
    authentication::change_password(*user_id, form.0.new_password, &pool)
        .await
        .map_err(e500)?;
    FlashMessage::error("Your password has been changed.").send();
    Ok(see_other("/admin/password"))
}
