use crate::{error::AppError, models::*};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_user(pool: &PgPool, new_user: CreateUser) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, email, department)
        VALUES ($1, $2, $3, $4)
        RETURNING id, username, email, department, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(new_user.username)
    .bind(new_user.email)
    .bind(new_user.department)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_all_users(
    pool: &PgPool,
    page: i64,
    per_page: i64,
) -> Result<PaginatedResponse<User>, AppError> {
    let offset = (page - 1) * per_page;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, department, created_at, updated_at
        FROM users
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(PaginatedResponse::new(users, total.0, page, per_page))
}

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, department, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))
}

pub async fn update_user(
    pool: &PgPool,
    user_id: Uuid,
    update: UpdateUser,
) -> Result<User, AppError> {
    sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET
            username   = COALESCE($1, username),
            email      = COALESCE($2, email),
            department = COALESCE($3, department),
            updated_at = NOW()
        WHERE id = $4
        RETURNING id, username, email, department, created_at, updated_at
        "#,
    )
    .bind(update.username)
    .bind(update.email)
    .bind(update.department)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))
}

pub async fn delete_user(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // Unassign any laptops belonging to this user before deletion.
    sqlx::query(
        r#"
        UPDATE laptops
        SET assigned_to = NULL, status = 'available', updated_at = NOW()
        WHERE assigned_to = $1
        "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        // Returning NotFound will drop the transaction, rolling it back.
        return Err(AppError::NotFound(format!("User {} not found", user_id)));
    }

    tx.commit().await?;
    Ok(())
}
