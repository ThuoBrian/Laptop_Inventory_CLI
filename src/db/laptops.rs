use crate::{db::users::get_user_by_id, error::AppError, models::*};
use sqlx::PgPool;
use uuid::Uuid;

const VALID_STATUSES: &[&str] = &["available", "assigned", "in_repair", "retired"];

fn validate_status(status: &str) -> Result<(), AppError> {
    if !VALID_STATUSES.contains(&status) {
        return Err(AppError::BadRequest(format!(
            "Invalid status '{}'. Must be one of: {}",
            status,
            VALID_STATUSES.join(", ")
        )));
    }
    Ok(())
}

pub async fn create_laptop(pool: &PgPool, new_laptop: CreateLaptop) -> Result<Laptop, AppError> {
    let laptop = sqlx::query_as::<_, Laptop>(
        r#"
        INSERT INTO laptops (id, brand, model, serial_number, purchase_date)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, brand, model, serial_number, status, assigned_to,
                  purchase_date, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(new_laptop.brand)
    .bind(new_laptop.model)
    .bind(new_laptop.serial_number)
    .bind(new_laptop.purchase_date)
    .fetch_one(pool)
    .await?;

    Ok(laptop)
}

pub async fn get_all_laptops(
    pool: &PgPool,
    status_filter: Option<String>,
) -> Result<Vec<Laptop>, AppError> {
    if let Some(ref s) = status_filter {
        validate_status(s)?;
    }

    let laptops = match status_filter {
        Some(status) => {
            sqlx::query_as::<_, Laptop>(
                r#"
                SELECT id, brand, model, serial_number, status, assigned_to,
                       purchase_date, created_at, updated_at
                FROM laptops
                WHERE status = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(status)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, Laptop>(
                r#"
                SELECT id, brand, model, serial_number, status, assigned_to,
                       purchase_date, created_at, updated_at
                FROM laptops
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(pool)
            .await?
        }
    };

    Ok(laptops)
}

pub async fn get_laptop_by_id(pool: &PgPool, laptop_id: Uuid) -> Result<Laptop, AppError> {
    sqlx::query_as::<_, Laptop>(
        r#"
        SELECT id, brand, model, serial_number, status, assigned_to,
               purchase_date, created_at, updated_at
        FROM laptops
        WHERE id = $1
        "#,
    )
    .bind(laptop_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Laptop {} not found", laptop_id)))
}

pub async fn update_laptop(
    pool: &PgPool,
    laptop_id: Uuid,
    update: UpdateLaptop,
) -> Result<Laptop, AppError> {
    if let Some(ref s) = update.status {
        validate_status(s)?;
        if s == "assigned" {
            return Err(AppError::BadRequest(
                "Use POST /laptops/{id}/assign to assign a laptop to a user.".to_string(),
            ));
        }
    }

    sqlx::query_as::<_, Laptop>(
        r#"
        UPDATE laptops
        SET
            brand         = COALESCE($1, brand),
            model         = COALESCE($2, model),
            serial_number = COALESCE($3, serial_number),
            status        = COALESCE($4, status),
            purchase_date = COALESCE($5, purchase_date),
            updated_at    = NOW()
        WHERE id = $6
        RETURNING id, brand, model, serial_number, status, assigned_to,
                  purchase_date, created_at, updated_at
        "#,
    )
    .bind(update.brand)
    .bind(update.model)
    .bind(update.serial_number)
    .bind(update.status)
    .bind(update.purchase_date)
    .bind(laptop_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Laptop {} not found", laptop_id)))
}

pub async fn delete_laptop(pool: &PgPool, laptop_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM laptops WHERE id = $1")
        .bind(laptop_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Laptop {} not found", laptop_id)));
    }

    Ok(())
}

pub async fn assign_laptop(
    pool: &PgPool,
    laptop_id: Uuid,
    user_id: Uuid,
) -> Result<Laptop, AppError> {
    // Verify the target user exists.
    get_user_by_id(pool, user_id).await?;

    // Verify the laptop is available.
    let laptop = get_laptop_by_id(pool, laptop_id).await?;
    if laptop.status != "available" {
        return Err(AppError::BadRequest(format!(
            "Laptop {} cannot be assigned — current status is '{}'.",
            laptop_id, laptop.status
        )));
    }

    sqlx::query_as::<_, Laptop>(
        r#"
        UPDATE laptops
        SET assigned_to = $1, status = 'assigned', updated_at = NOW()
        WHERE id = $2
        RETURNING id, brand, model, serial_number, status, assigned_to,
                  purchase_date, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(laptop_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn unassign_laptop(pool: &PgPool, laptop_id: Uuid) -> Result<Laptop, AppError> {
    let laptop = get_laptop_by_id(pool, laptop_id).await?;
    if laptop.status != "assigned" {
        return Err(AppError::BadRequest(format!(
            "Laptop {} is not currently assigned (status: '{}').",
            laptop_id, laptop.status
        )));
    }

    sqlx::query_as::<_, Laptop>(
        r#"
        UPDATE laptops
        SET assigned_to = NULL, status = 'available', updated_at = NOW()
        WHERE id = $1
        RETURNING id, brand, model, serial_number, status, assigned_to,
                  purchase_date, created_at, updated_at
        "#,
    )
    .bind(laptop_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}
