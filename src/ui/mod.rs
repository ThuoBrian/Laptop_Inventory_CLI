pub mod templates;
pub mod pages;
pub mod fragments;

use crate::error::AppError;
use actix_web::HttpResponse;
use minijinja::Environment;

pub fn render_template(
    env: &Environment<'static>,
    name: &str,
    ctx: minijinja::Value,
) -> Result<HttpResponse, AppError> {
    let tmpl = env
        .get_template(name)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let html = tmpl
        .render(ctx)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
