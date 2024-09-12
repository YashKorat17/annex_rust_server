#[path ="routes/cust.rs"] pub mod cust;
#[path ="routes/estimate.rs"] pub mod estimate;
#[path ="routes/payment.rs"] pub mod payment;
extern crate dotenv;

use dotenv::dotenv;
use estimate::{get_estimate, get_estimate_id};
use payment::{get_payment, get_payment_id};
use std::env;
use actix_cors::Cors;
use actix_web::{ http, web, App, HttpServer};
use mongodb::{
    options::ClientOptions, Client
};
use cust::{check_username, get_all_customers, get_all_customers_estimate, get_all_customers_payments, get_customers};


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
            .service(get_customers)
            .service(cust::test)
            .service(check_username)
            .service(get_all_customers)
            .service(get_all_customers_estimate)
            .service(get_all_customers_payments)
            .service(
                web::scope("/api/v1/estimate")
                .service(get_estimate_id)
                .service(get_estimate)
        )
        .service(
            web::scope("/api/v1/payment")
            .service(get_payment_id)
            .service(get_payment)   
        )
    })
    .bind(format!("{}:{}",env::var("RUST_HOST").unwrap(), env::var("RUST_PORT").unwrap()))?
    .workers(num_cpus::get())
    .run()

    .await
}