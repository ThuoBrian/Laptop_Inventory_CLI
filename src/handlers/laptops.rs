use crate::{databases::create_laptop, error::AppError, models::Laptop};
use actix_web::{HttpResponse, Responder, post, web};

#[post("/laptops")]
pub async fn create_laptop(
    pool: &sqlx::PgPool,
    new_laptop: web::Json<Laptop>,
) -> Result<impl Responder, AppError> {
    let laptop = create_laptop(&pool, new_laptop.into_inner()).await?;
    Ok(HttpResponse::Created().json(laptop))
}
