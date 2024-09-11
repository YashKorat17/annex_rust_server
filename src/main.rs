#[path ="routes/cust.rs"] pub mod cust;
extern crate dotenv;

use dotenv::dotenv;
use std::env;
use actix_cors::Cors;
use actix_web::{ http, web, App, HttpServer};
use mongodb::{
    options::ClientOptions, Client
};
use cust::{check_username, get_all_customers, get_all_customers_estimate, get_customers, test};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();  
    
    let client_options: ClientOptions = ClientOptions::parse(
        env::var("MONGODB_URL").unwrap_or("mongodb://localhost:27017".to_string())
    ).await.unwrap();

    let client: Client = Client::with_options(client_options).unwrap();
    HttpServer::new(move || {
        let cors = Cors::default()
                .allow_any_origin()
                .supports_credentials()
                .allowed_methods(vec!["GET", "POST" , "OPTIONS"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT, http::header::CONTENT_TYPE, http::header::COOKIE])
                .max_age(600);

        App::new()
            .wrap(cors)
            .app_data(
                web::Data::new(client.clone())
            )
            .service(test)
            .service(get_customers)
            .service(check_username)
            .service(get_all_customers)
            .service(get_all_customers_estimate)
    })
    .bind(format!("{}:{}",env::var("RUST_HOST").unwrap(), env::var("RUST_PORT").unwrap()))?
    .workers(num_cpus::get())
    .run()

    .await
}