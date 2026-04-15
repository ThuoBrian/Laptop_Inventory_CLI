mod db;
mod error;
mod handlers;
mod models;
mod request_id;
mod ui;
mod validation;

use actix_web::{App, HttpServer, middleware, web};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let max_connections = env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(10);

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "5342".to_string());
    let bind_addr = format!("{}:{}", host, port);

    log::info!("Database connection established.");
    log::info!("Server starting at http://{}", bind_addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(ui::templates::create_environment()))
            .app_data(web::JsonConfig::default().limit(1024 * 1024))
            .app_data(web::FormConfig::default().limit(1024 * 1024))
            .wrap(request_id::RequestId)
            .wrap(middleware::Logger::default())
            // ── Static files ─────────────────────────────────────────
            .service(actix_files::Files::new("/static", "./static"))
            // ── API: Users ───────────────────────────────────────────
            .service(handlers::users::create_user)
            .service(handlers::users::get_all_users)
            .service(handlers::users::get_user)
            .service(handlers::users::update_user)
            .service(handlers::users::delete_user)
            // ── API: Laptops ──────────────────────────────────────────
            .service(handlers::laptops::create_laptop)
            .service(handlers::laptops::get_all_laptops)
            .service(handlers::laptops::get_laptop)
            .service(handlers::laptops::update_laptop)
            .service(handlers::laptops::delete_laptop)
            .service(handlers::laptops::assign_laptop)
            .service(handlers::laptops::unassign_laptop)
            // ── Web UI: Pages ────────────────────────────────────────
            .service(ui::pages::dashboard)
            .service(ui::pages::laptops_page)
            .service(ui::pages::users_page)
            // ── Web UI: Laptop fragments ─────────────────────────────
            .service(ui::fragments::create_laptop_form)
            .service(ui::fragments::edit_laptop_form)
            .service(ui::fragments::update_laptop_form)
            .service(ui::fragments::delete_laptop_form)
            .service(ui::fragments::assign_laptop_form_get)
            .service(ui::fragments::assign_laptop_form)
            .service(ui::fragments::unassign_laptop_form)
            // ── Web UI: User fragments ───────────────────────────────
            .service(ui::fragments::create_user_form)
            .service(ui::fragments::edit_user_form)
            .service(ui::fragments::update_user_form)
            .service(ui::fragments::delete_user_form)
    })
    .bind(&bind_addr)?
    .run()
    .await
}