// src/main.rs
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use sea_orm::{Database, DatabaseConnection};
use std::env;
mod handlers;
mod models;
mod routes;
mod services;
mod utils;

//===============================
// Main Function
//===============================
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let db: DatabaseConnection = Database::connect(&db_url)
        .await
        .expect("Failed to connect to the database");
    println!("✅ Database connected");
    let host: String = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a valid u16");
    println!("🚀 Server running at http://{}:{}", host, port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(routes::auth_route::configure_routes)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
