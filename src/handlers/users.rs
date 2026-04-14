use crate::{db, error::AppError, models::*, validation};
use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use sqlx::PgPool;
use uuid::Uuid;

#[post("/users")]
pub async fn create_user(
    pool: web::Data<PgPool>,
    body: web::Json<CreateUser>,
) -> Result<impl Responder, AppError> {
    let body = body.into_inner();
    let username = validation::validate_required_string(&body.username, "Username")?;
    let email = validation::validate_email(&body.email)?;
    let department = validation::validate_required_string(&body.department, "Department")?;
    let validated = CreateUser { username, email, department };
    let user = db::users::create_user(&pool, validated).await?;
    Ok(HttpResponse::Created().json(user))
}

#[get("/users")]
pub async fn get_all_users(pool: web::Data<PgPool>) -> Result<impl Responder, AppError> {
    let users = db::users::get_all_users(&pool).await?;
    Ok(HttpResponse::Ok().json(users))
}

#[get("/users/{id}")]
pub async fn get_user(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let user = db::users::get_user_by_id(&pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(user))
}

#[put("/users/{id}")]
pub async fn update_user(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateUser>,
) -> Result<impl Responder, AppError> {
    let body = body.into_inner();
    let username = validation::validate_optional_string(body.username.as_ref(), "Username")?;
    let email = match body.email.as_ref() {
        Some(e) => Some(validation::validate_email(e)?),
        None => None,
    };
    let department = validation::validate_optional_string(body.department.as_ref(), "Department")?;
    let validated = UpdateUser { username, email, department };
    let user = db::users::update_user(&pool, path.into_inner(), validated).await?;
    Ok(HttpResponse::Ok().json(user))
}

#[delete("/users/{id}")]
pub async fn delete_user(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    db::users::delete_user(&pool, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
