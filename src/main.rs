#[path ="routes/cust.rs"] pub mod cust;
#[path ="routes/product.rs"] pub mod product;
#[path ="routes/estimate.rs"] pub mod estimate;
#[path ="routes/invoice.rs"] pub mod inv;
#[path ="routes/payment.rs"] pub mod payment;
#[path ="routes/storage.rs"] pub mod storage;
extern crate dotenv;

use dotenv::dotenv;
use estimate::{get_estimate, get_estimate_id, search_estimate};
use payment::{get_payment, get_payment_id, search_payments};
use product::{get_category, search_product};
use storage::{get_info, get_media, get_temp_media};
use std::env;
use actix_cors::Cors;
use actix_web::{ http, web, App, HttpServer};
use mongodb::{
    options::ClientOptions, Client
};
use cust::{check_username, get_all_customers, get_all_customers_estimate, get_all_customers_payments, get_customers, search_customers,get_customer_statement};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();  
    let client_options: ClientOptions = ClientOptions::parse(
        env::var("MONGODB_URL").unwrap_or("mongodb://localhost:8145".to_string())
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
            .service(search_customers)
            .service(get_customer_statement)
            .service(
                web::scope("/api/v1/product")
                .service(search_product)
                .service(get_category)
            )
            .service(
                web::scope("/api/v1/estimate")
                .service(get_estimate_id)
                .service(get_estimate)
                .service(search_estimate)
        )
        .service(
            web::scope("/api/v1/storage")
            .service(get_media)
            .service(get_info)
            .service(get_temp_media)
        )
        .service(
            web::scope("/api/v1/payment")
            .service(get_payment_id)
            .service(get_payment)   
            .service(search_payments)
        )
        .service(
            web::scope("/api/v1/invoice")
            .service(inv::get_inv_id)
            .service(inv::get_inv)
        )
    })
    .bind(format!("{}:{}",env::var("RUST_HOST").unwrap(), env::var("RUST_PORT").unwrap()))?
    .workers(num_cpus::get())
    .run()
    .await
}