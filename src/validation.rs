use crate::error::AppError;

const MAX_LEN: usize = 100;

pub fn validate_required_string(value: &str, field_name: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(format!(
            "{} cannot be empty",
            field_name
        )));
    }
    if trimmed.len() > MAX_LEN {
        return Err(AppError::BadRequest(format!(
            "{} cannot exceed {} characters",
            field_name, MAX_LEN
        )));
    }
    Ok(trimmed.to_string())
}

pub fn validate_optional_string(
    value: Option<&String>,
    field_name: &str,
) -> Result<Option<String>, AppError> {
    match value {
        Some(s) if !s.trim().is_empty() => {
            let trimmed = s.trim();
            if trimmed.len() > MAX_LEN {
                return Err(AppError::BadRequest(format!(
                    "{} cannot exceed {} characters",
                    field_name, MAX_LEN
                )));
            }
            Ok(Some(trimmed.to_string()))
        }
        Some(_) => Err(AppError::BadRequest(format!(
            "{} cannot be empty",
            field_name
        ))),
        None => Ok(None),
    }
}

pub fn validate_email(email: &str) -> Result<String, AppError> {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("Email cannot be empty".to_string()));
    }
    if trimmed.len() > MAX_LEN {
        return Err(AppError::BadRequest(
            "Email cannot exceed 100 characters".to_string(),
        ));
    }
    if !trimmed.contains('@') || !trimmed.split('@').nth(1).map_or(false, |d| d.contains('.')) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }
    Ok(trimmed.to_string())
}