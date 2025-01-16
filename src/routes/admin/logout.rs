use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;

use crate::{authentication::UserId, session_stare::TypedSession, utils::see_other};

pub async fn log_out(
    session: TypedSession,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    user_id.into_inner();
    session.log_out();
    FlashMessage::info("You have successfully logged out.").send();
    Ok(see_other("/login"))
}
