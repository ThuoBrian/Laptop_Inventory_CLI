mod db;
mod error;
mod handlers;
mod models;

use actix_web::{App, HttpServer, middleware, web};
use sqlx::PgPool;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    log::info!("Database connection established.");
    log::info!("Server starting at http://127.0.0.1:5342");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            // ── Users ────────────────────────────────────────────────────
            .service(handlers::users::create_user)
            .service(handlers::users::get_all_users)
            .service(handlers::users::get_user)
            .service(handlers::users::update_user)
            .service(handlers::users::delete_user)
            // ── Laptops ──────────────────────────────────────────────────
            .service(handlers::laptops::create_laptop)
            .service(handlers::laptops::get_all_laptops)
            .service(handlers::laptops::get_laptop)
            .service(handlers::laptops::update_laptop)
            .service(handlers::laptops::delete_laptop)
            .service(handlers::laptops::assign_laptop)
            .service(handlers::laptops::unassign_laptop)
    })
    .bind("127.0.0.1:5342")?
    .run()
    .await
}
