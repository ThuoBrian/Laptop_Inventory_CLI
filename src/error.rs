use actix_web::{HttpResponse, error::ResponseError, http::StatusCode};
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
        let body = json!({ "error": self.to_string() });
        HttpResponse::build(self.status_code()).json(body)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::Database(db_err) => {
                // PostgreSQL unique-constraint violation
                if db_err.code().as_deref() == Some("23505") {
                    AppError::Conflict(db_err.message().to_string())
                } else {
                    AppError::Database(err.to_string())
                }
            }
            _ => AppError::Database(err.to_string()),
        }
    }
}
