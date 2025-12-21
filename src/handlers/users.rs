use crate::{databases, error::AppError, models::*};
use actix_web::{HttpResponse, Responder, get, post, web};
use sqlx::PgPool;

#[post("/users")]
pub async fn create_user(
    pool: web::Data<PgPool>,
    new_user: web::Json<CreateUser>,
) -> Result<impl Responder, AppError> {
    let user = databases::create_user(&pool, new_user.into_inner()).await?;
    Ok(HttpResponse::Created().json(user))
}
#[get("/users")]
pub async fn get_all_users(pool: web::Data<PgPool>) -> Result<impl Responder, AppError> {
    let users = databases::get_all_users(&pool).await?;
    Ok(HttpResponse::Ok().json(users))
}
