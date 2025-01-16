use std::fmt::Write;

use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

#[tracing::instrument(name = "Get newsletter form", skip(flash_messages))]
pub async fn newsletter_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut message_html = String::new();
    for m in flash_messages.iter() {
        writeln!(message_html, "<p><i>{}</i></p>", m.content()).unwrap()
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Publish Newsletters Issue</title>
        </head>
        <body>
            {message_html}
            <form action="/admin/newsletters" method="post">
                <label>Title
                <input
                    type="text"
                    placeholder="Enter the title"
                    name="title"
                />
                </label>
                <br>
                <label>
                Plain text content:
                <br>
                <textarea
                    placeholder="Enter the content in HTML text."
                    name="text_content"
                    rows="20"
                    cols="50"
                ></textarea>
                </label>
                <br>
                <label>
                Plain text content:
                <br>
                <textarea
                    placeholder="Enter the content in HTML text."
                    name="html_content"
                    rows="20"
                    cols="50"
                ></textarea>
                </label>
                <button type="submit">Pubish</button>
            </form>
            <p><a href="/admin/dashboard">&lt;- Back</a></p>
        </body>
        </html>
        "#,
        )))
}
