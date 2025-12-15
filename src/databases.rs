use crate::{error::AppError, models::*};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_user(pool: &PgPool, new_user: CreateUser) -> Result<User, AppError> {
    let result = sqlx::query!(
        User,
        r#"
        INSERT INTO users (id, username, email)
        VALUES ($1, $2, $3)
        RETURNING id, username, email
        "#,
        Uuid::new_v4(),
        new_user.username,
        new_user.email
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;

    Ok(result)
}

pub async fn get_all_users(pool: &PgPool) -> Result<Vec<User>, AppError> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email
        FROM users
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    Ok(users)
}
pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::RowNotFound => {
            AppError::NotFound(format!("User with id {} not found", user_id))
        }
        _ => AppError::from(err),
    })?;

    Ok(user)
}
