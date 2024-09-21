#[path ="routes/cust.rs"] pub mod cust;
#[path ="routes/product.rs"] pub mod product;
#[path ="routes/estimate.rs"] pub mod estimate;
#[path ="routes/payment.rs"] pub mod payment;
extern crate dotenv;

use dotenv::dotenv;
use estimate::{get_estimate, get_estimate_id, search_estimate};
use payment::{get_payment, get_payment_id, search_payments};
use product::search_product;
use std::{env, fs::File, io::BufReader};
use actix_cors::Cors;
use actix_web::{ http, web, App, HttpServer};
use mongodb::{
    options::ClientOptions, Client
};
use cust::{check_username, get_all_customers, get_all_customers_estimate, get_all_customers_payments, get_customers, search_customers,get_customer_statement};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();  

    rustls::crypto::aws_lc_rs::default_provider()
    .install_default()
    .unwrap();

let mut certs_file = BufReader::new(File::open("/root/annex_rust_server/ssl/cert.pem").unwrap());
let mut key_file = BufReader::new(File::open("/root/annex_rust_server/ssl/key.pem").unwrap());

// load TLS certs and key
// to create a self-signed temporary cert for testing:
// `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
let tls_certs = rustls_pemfile::certs(&mut certs_file)
    .collect::<Result<Vec<_>, _>>()
    .unwrap();
let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
    .next()
    .unwrap()
    .unwrap();

// set up TLS config options
let tls_config = rustls::ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
    .unwrap();


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
            .service(search_customers)
            .service(get_customer_statement)
            .service(
                web::scope("/api/v1/product")
                .service(search_product)
            )
            .service(
                web::scope("/api/v1/estimate")
                .service(get_estimate_id)
                .service(get_estimate)
                .service(search_estimate)
        )
        .service(
            web::scope("/api/v1/payment")
            .service(get_payment_id)
            .service(get_payment)   
            .service(search_payments)
        )
    })
    .bind_rustls_0_23(format!("{}:{}",env::var("RUST_HOST").unwrap(), env::var("RUST_PORT").unwrap()),tls_config)?
    .workers(num_cpus::get())
    .run()

    .await
}