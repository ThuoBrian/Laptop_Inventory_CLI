use crate::{db, error::AppError, models::*};
use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct StatusQuery {
    pub status: Option<String>,
}

#[post("/laptops")]
pub async fn create_laptop(
    pool: web::Data<PgPool>,
    body: web::Json<CreateLaptop>,
) -> Result<impl Responder, AppError> {
    let laptop = db::laptops::create_laptop(&pool, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(laptop))
}

#[get("/laptops")]
pub async fn get_all_laptops(
    pool: web::Data<PgPool>,
    query: web::Query<StatusQuery>,
) -> Result<impl Responder, AppError> {
    let laptops = db::laptops::get_all_laptops(&pool, query.into_inner().status).await?;
    Ok(HttpResponse::Ok().json(laptops))
}

#[get("/laptops/{id}")]
pub async fn get_laptop(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let laptop = db::laptops::get_laptop_by_id(&pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(laptop))
}

#[put("/laptops/{id}")]
pub async fn update_laptop(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateLaptop>,
) -> Result<impl Responder, AppError> {
    let laptop = db::laptops::update_laptop(&pool, path.into_inner(), body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(laptop))
}

#[delete("/laptops/{id}")]
pub async fn delete_laptop(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    db::laptops::delete_laptop(&pool, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[post("/laptops/{id}/assign")]
pub async fn assign_laptop(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    body: web::Json<AssignLaptop>,
) -> Result<impl Responder, AppError> {
    let laptop = db::laptops::assign_laptop(&pool, path.into_inner(), body.user_id).await?;
    Ok(HttpResponse::Ok().json(laptop))
}

#[post("/laptops/{id}/unassign")]
pub async fn unassign_laptop(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let laptop = db::laptops::unassign_laptop(&pool, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(laptop))
}
