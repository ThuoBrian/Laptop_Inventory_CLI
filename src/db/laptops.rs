use crate::{db::users::get_user_by_id, error::AppError, models::*};
use sqlx::PgPool;
use uuid::Uuid;

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
    status_filter: Option<LaptopStatus>,
    page: i64,
    per_page: i64,
) -> Result<PaginatedResponse<Laptop>, AppError> {
    let offset = (page - 1) * per_page;

    let laptops = match &status_filter {
        Some(status) => {
            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM laptops WHERE status = $1",
            )
            .bind(status.to_string())
            .fetch_one(pool)
            .await?;

            let rows = sqlx::query_as::<_, Laptop>(
                r#"
                SELECT id, brand, model, serial_number, status, assigned_to,
                       purchase_date, created_at, updated_at
                FROM laptops
                WHERE status = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(status.to_string())
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?;

            PaginatedResponse::new(rows, total.0, page, per_page)
        }
        None => {
            let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM laptops")
                .fetch_one(pool)
                .await?;

            let rows = sqlx::query_as::<_, Laptop>(
                r#"
                SELECT id, brand, model, serial_number, status, assigned_to,
                       purchase_date, created_at, updated_at
                FROM laptops
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?;

            PaginatedResponse::new(rows, total.0, page, per_page)
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
    .ok_or_else(|| AppError::NotFound("Laptop not found".to_string()))
}

pub async fn update_laptop(
    pool: &PgPool,
    laptop_id: Uuid,
    update: UpdateLaptop,
) -> Result<Laptop, AppError> {
    if let Some(ref s) = update.status {
        if s == &LaptopStatus::Assigned {
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
    .ok_or_else(|| AppError::NotFound("Laptop not found".to_string()))
}

pub async fn delete_laptop(pool: &PgPool, laptop_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM laptops WHERE id = $1")
        .bind(laptop_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Laptop not found".to_string()));
    }

    Ok(())
}

pub async fn assign_laptop(
    pool: &PgPool,
    laptop_id: Uuid,
    user_id: Uuid,
) -> Result<Laptop, AppError> {
    // Verify the target user exists up-front for a friendlier error than
    // a raw FK violation.
    get_user_by_id(pool, user_id).await?;

    let updated = sqlx::query_as::<_, Laptop>(
        r#"
        UPDATE laptops
        SET assigned_to = $1, status = 'assigned', updated_at = NOW()
        WHERE id = $2 AND status = 'available'
        RETURNING id, brand, model, serial_number, status, assigned_to,
                  purchase_date, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(laptop_id)
    .fetch_optional(pool)
    .await?;

    match updated {
        Some(laptop) => Ok(laptop),
        None => {
            // Either the laptop is missing or its status changed since the
            // caller last saw it — re-read to produce an accurate error.
            let laptop = get_laptop_by_id(pool, laptop_id).await?;
            Err(AppError::BadRequest(format!(
                "Laptop {} cannot be assigned — current status is '{}'.",
                laptop_id, laptop.status
            )))
        }
    }
}

pub async fn unassign_laptop(pool: &PgPool, laptop_id: Uuid) -> Result<Laptop, AppError> {
    let updated = sqlx::query_as::<_, Laptop>(
        r#"
        UPDATE laptops
        SET assigned_to = NULL, status = 'available', updated_at = NOW()
        WHERE id = $1 AND status = 'assigned'
        RETURNING id, brand, model, serial_number, status, assigned_to,
                  purchase_date, created_at, updated_at
        "#,
    )
    .bind(laptop_id)
    .fetch_optional(pool)
    .await?;

    match updated {
        Some(laptop) => Ok(laptop),
        None => {
            let laptop = get_laptop_by_id(pool, laptop_id).await?;
            Err(AppError::BadRequest(format!(
                "Laptop {} is not currently assigned (status: '{}').",
                laptop_id, laptop.status
            )))
        }
    }
}

// ── Web UI queries (with assignee name) ───────────────────────────────────

pub async fn get_all_laptops_with_assignee(
    pool: &PgPool,
    status_filter: Option<LaptopStatus>,
    page: i64,
    per_page: i64,
) -> Result<PaginatedResponse<LaptopWithAssignee>, AppError> {
    let offset = (page - 1) * per_page;

    let result = match &status_filter {
        Some(status) => {
            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM laptops WHERE status = $1",
            )
            .bind(status.to_string())
            .fetch_one(pool)
            .await?;

            let rows = sqlx::query_as::<_, LaptopWithAssignee>(
                r#"
                SELECT l.id, l.brand, l.model, l.serial_number, l.status,
                       l.assigned_to, u.username AS assignee_name,
                       l.purchase_date, l.created_at, l.updated_at
                FROM laptops l
                LEFT JOIN users u ON l.assigned_to = u.id
                WHERE l.status = $1
                ORDER BY l.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(status.to_string())
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?;

            PaginatedResponse::new(rows, total.0, page, per_page)
        }
        None => {
            let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM laptops")
                .fetch_one(pool)
                .await?;

            let rows = sqlx::query_as::<_, LaptopWithAssignee>(
                r#"
                SELECT l.id, l.brand, l.model, l.serial_number, l.status,
                       l.assigned_to, u.username AS assignee_name,
                       l.purchase_date, l.created_at, l.updated_at
                FROM laptops l
                LEFT JOIN users u ON l.assigned_to = u.id
                ORDER BY l.created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?;

            PaginatedResponse::new(rows, total.0, page, per_page)
        }
    };

    Ok(result)
}

pub async fn count_laptops(pool: &PgPool) -> Result<i64, AppError> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM laptops")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn count_laptops_by_status(pool: &PgPool) -> Result<Vec<(String, i64)>, AppError> {
    let counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT status, COUNT(*) FROM laptops GROUP BY status ORDER BY status",
    )
    .fetch_all(pool)
    .await?;
    Ok(counts)
}

pub async fn get_recent_laptops(pool: &PgPool, limit: i64) -> Result<Vec<LaptopWithAssignee>, AppError> {
    let result = sqlx::query_as::<_, LaptopWithAssignee>(
        "SELECT l.id, l.brand, l.model, l.serial_number, l.status, l.assigned_to, \
         u.username AS assignee_name, l.purchase_date, l.created_at, l.updated_at \
         FROM laptops l \
         LEFT JOIN users u ON l.assigned_to = u.id \
         ORDER BY l.updated_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(result)
}
