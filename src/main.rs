// This file sets up the Actix web server, connects to the PostgreSQL database,
// and configures the routes and middleware.
mod databases;
mod error;
mod handlers;
mod models;

// Bringing in necessary crates for the web server and database connection..
use actix_web::{App, HttpServer, middleware, web};
use sqlx::PgPool;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // To read environment variables from a .env file.
    dotenv::dotenv().ok();
    env_logger::init();

    // Setting up the database connection pool.
    // The DATABASE_URL should be set in the environment variables.
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create pool connection");

    // Starting the HTTP server.
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(handlers::users::create_user)
            .service(handlers::users::get_all_users)
    })
    .run()
    .await?;

    // The server is running on localhost:5342
    println!("Server running at http://localhost:5342/");
}
