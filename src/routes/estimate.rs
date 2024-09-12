#[path = "../models/auth.rs"]
pub mod auth;
#[path = "../models/estimate.rs"]
pub mod e_model;


use actix_web::{get, http::header, post, web, HttpRequest, HttpResponse, Responder};
use auth::{validate_token, User};
use bson::{doc, Document};
use chrono::Datelike;
use e_model::{InvoiceIdReturn, Search};
use futures::TryStreamExt;
use mongodb::{Client, Collection};
use std::env;



#[get("/get/id/{month}/{year}")]
pub async fn get_estimate_id(
    mut data: web::Path<(i32, i32)>,
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

    let current_year:i32 = chrono::Utc::now().year();


    if data.1 < current_year - 10 || data.1 > current_year + 2 {
        return HttpResponse::BadRequest().json(doc! {
            "status": false,
            "msg": "Invalid year"
        });
    }

    if data.0 < 1 || data.0 > 12 {
        return HttpResponse::BadRequest().json(doc! {
            "status": false,
            "msg": "Invalid month"
        });
    }

    if data.0 < 4 {
        data.1 = data.1 - 1;
    }

    let coll: Collection<InvoiceIdReturn> = client
        .database(&env::var("DATABASE_NAME").unwrap())
        .collection("annex_inc_estimate_invoice");


    let cursor: Vec<InvoiceIdReturn> = coll.find(
        doc! {
            "u_id": &b.1,
            "date": doc!{
                "$gte": format!("{}-04-01",data.1),
                "$lt": format!("{}-03-31", data.1+1)
            }
        },
    ).projection(doc! {
        "inv_num": 1,
        "_id":0
    }).sort(doc! {
        "inv_num": -1
    }).limit(1).await.unwrap().try_collect::<Vec<_>>().await.unwrap();
    if cursor.is_empty() {
        return HttpResponse::Ok().json(
            doc! {
                "n" : 1 
            }
        );
    }

    HttpResponse::Ok().json(
        doc! {
            "n" : cursor[0].inv_num + 1
        }
    )

} 

#[get("/{id}")]
pub async fn get_estimate(
    data: web::Path<String>,
    client: web::Data<Client>,
    req: HttpRequest,
) -> impl Responder {
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

    let coll: Collection<Document> = client
        .database(&env::var("DATABASE_NAME").unwrap())
        .collection("annex_inc_estimate_invoice");

    let cursor: Option<Document> = coll.find_one(
        doc! {
            "u_id": &b.1,
            "_id": &data.to_string()
        },
    ).projection(doc! {
        "cre_at": 0,
        "upd_at": 0,
        "is_del": 0,
        "u_id": 0,
        "_id": 0,
    }).await.unwrap();


    HttpResponse::Ok().json(cursor)
   
}

#[post("/search")]
pub async fn search_estimate(
    data: web::Json<Search>,
    client: web::Data<Client>,
    req: HttpRequest,
) -> impl Responder {
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

    

    HttpResponse::Ok().json(doc!{
        "status": true
    })
}