use crate::{db, error::AppError, models::*, validation};
use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use sqlx::PgPool;
use uuid::Uuid;

#[post("/laptops")]
pub async fn create_laptop(
    pool: web::Data<PgPool>,
    body: web::Json<CreateLaptop>,
) -> Result<impl Responder, AppError> {
    let body = body.into_inner();
    let brand = validation::validate_required_string(&body.brand, "Brand")?;
    let model = validation::validate_required_string(&body.model, "Model")?;
    let serial_number = validation::validate_required_string(&body.serial_number, "Serial number")?;
    let validated = CreateLaptop { brand, model, serial_number, purchase_date: body.purchase_date };
    let laptop = db::laptops::create_laptop(&pool, validated).await?;
    Ok(HttpResponse::Created().json(laptop))
}

#[get("/laptops")]
pub async fn get_all_laptops(
    pool: web::Data<PgPool>,
    query: web::Query<LaptopListQuery>,
) -> Result<impl Responder, AppError> {
    let page = clamp_page(query.page);
    let per_page = clamp_per_page(query.per_page);
    let laptops = db::laptops::get_all_laptops(&pool, query.into_inner().status, page, per_page).await?;
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
    let body = body.into_inner();
    let brand = validation::validate_optional_string(body.brand.as_ref(), "Brand")?;
    let model = validation::validate_optional_string(body.model.as_ref(), "Model")?;
    let serial_number = validation::validate_optional_string(body.serial_number.as_ref(), "Serial number")?;
    let status = body.status;
    let purchase_date = body.purchase_date;
    let validated = UpdateLaptop { brand, model, serial_number, status, purchase_date };
    let laptop = db::laptops::update_laptop(&pool, path.into_inner(), validated).await?;
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
