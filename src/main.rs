mod databases;
mod error;
mod handlers;
mod models;

use actix_web::{App, HttpServer, middleware, web};
use sqlx::PgPool;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create pool connection");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(handlers::users::create_user)
            .service(handlers::users::get_all_users)
    })
    .run()
    .await
}
