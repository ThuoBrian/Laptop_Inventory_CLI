use actix_web::{
    HttpRequest, HttpResponse, Responder, body::BoxBody, error::ResponseError, http::StatusCode, web,
};
use minijinja::Environment;
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Conflict(String),
    Database(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg)   => write!(f, "{}", msg),
            AppError::BadRequest(msg) => write!(f, "{}", msg),
            AppError::Conflict(msg)   => write!(f, "{}", msg),
            AppError::Database(msg)   => write!(f, "{}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_)   => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_)   => StatusCode::CONFLICT,
            AppError::Database(_)   => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        if let AppError::Database(ref msg) = self {
            log::error!("Database error: {}", msg);
        }
        let message = match self {
            AppError::Database(_) => "Internal server error".to_string(),
            _ => self.to_string(),
        };
        let body = json!({ "error": message });
        HttpResponse::build(self.status_code()).json(body)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::Database(db_err) => {
                // PostgreSQL unique-constraint violation
                if db_err.code().as_deref() == Some("23505") {
                    log::warn!(
                        "Unique constraint violation (constraint={:?}): {}",
                        db_err.constraint(),
                        db_err.message()
                    );
                    let friendly = match db_err.constraint() {
                        Some("users_email_key") => "A user with this email already exists.",
                        Some("users_username_key") => "A user with this username already exists.",
                        Some("laptops_serial_number_key") => {
                            "A laptop with this serial number already exists."
                        }
                        _ => "A record with this value already exists.",
                    };
                    AppError::Conflict(friendly.to_string())
                } else {
                    AppError::Database(err.to_string())
                }
            }
            _ => AppError::Database(err.to_string()),
        }
    }
}

/// Responder wrapper for UI handlers: renders `AppError` as an HTML fragment
/// (so HTMX swaps a readable error into the DOM) instead of JSON.
///
/// Looks up the Minijinja environment from request app data; falls back to
/// the default JSON error response if it isn't registered.
pub struct UiResult(pub Result<HttpResponse, AppError>);

impl Responder for UiResult {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        match self.0 {
            Ok(r) => r,
            Err(e) => match req.app_data::<web::Data<Environment<'static>>>() {
                Some(env) => e.to_html(env.get_ref()),
                None => e.error_response(),
            },
        }
    }
}

impl AppError {
    pub fn to_html(&self, env: &minijinja::Environment) -> HttpResponse {
        let message = match self {
            AppError::Database(_) => "Internal server error".to_string(),
            _ => self.to_string(),
        };
        let html = env
            .get_template("partials/error.html")
            .and_then(|tmpl| {
                tmpl.render(minijinja::context! {
                    status_code => self.status_code().as_u16(),
                    message,
                })
            })
            .unwrap_or_else(|_| format!("Error {}: {}", self.status_code().as_u16(), message));
        HttpResponse::build(self.status_code())
            .content_type("text/html; charset=utf-8")
            .body(html)
    }
}
