#[path = "../models/auth.rs"]
pub mod auth;


use actix_web::{get, http::header, post, web, HttpRequest, HttpResponse, Responder};
use auth::validate_token;
use bson::{doc, Document};
use mongodb::{Client, Collection};
use std::env;


#[get("")]
pub async fn get_settings(
    client: web::Data<Client>,
    req: HttpRequest,
)
-> impl Responder {
    let token: &str = req.headers().get(header::AUTHORIZATION).and_then(
        |value| value.to_str().ok())
        .and_then(
            |value| value.strip_prefix("Bearer "))
        .unwrap_or("")
        .trim();
        
    let b: (bool, String) = validate_token(token, &client).await;

    if !b.0 {
        return HttpResponse::Unauthorized().finish();
    }

    let data: Collection<Document> = client
        .database(&env::var("DATABASE_NAME").unwrap())
        .collection("annex_inc_settings");

    let v = data
        .find_one(doc! {
            "u_id": b.1
        })
        .projection(doc! {
            "u_id": 0,
            "_id": 0,
        })
        .await
        .unwrap()
        .unwrap_or(doc! {});

    if v.is_empty() {
        return HttpResponse::Ok().json(
            doc! {
                "msg": "No data found",
            }
        );
    }

    HttpResponse::Ok().json(v)
}


