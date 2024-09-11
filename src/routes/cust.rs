#[path = "../models/auth.rs"]
pub mod auth;
#[path = "../models/cust.rs"]
pub mod cust_model;

use actix_web::{get, http::header, post, web, HttpRequest, HttpResponse, Responder};
use auth::{validate_token, User};
use bson::doc;
use cust_model::{AnnexIdCheckGstin, AnnexIdCheckUsername, AnnexResponse, Customer, GetInvoices, Users};
use futures::TryStreamExt;
use mongodb::{Client, Collection};
use std::env;


#[get("/test")]
pub async fn test() -> impl Responder {
    println!("Test");
    HttpResponse::Ok().json(doc! {
        "status": true
    })
}


#[get("/api/v1/customer/{id}")]
pub async fn get_customers(
    id: web::Path<String>,
    client: web::Data<Client>,
    req: HttpRequest,
) -> impl Responder {

    let token: &str = req.headers().get(header::COOKIE).unwrap().to_str().unwrap();
    let b: (bool, String) = validate_token(token, &client).await;

    if b.0 {
        let coll: Collection<Customer> = client
            .database(&env::var("DATABASE_NAME").unwrap())
            .collection("annex_inc_customers");

        let cursor: Option<Customer> = coll
            .find_one(doc! {
                "_id": &id.to_string(),
                "u_id": &b.1
            })
            .projection(doc! {
               "cre_at":0,
               "upd_at":0,
               "cls_bal":0,
               "cls_fine":0,
               "u_id":0
            })
            .await
            .unwrap();
        let customers: Customer = cursor.unwrap();
        HttpResponse::Ok().json(customers)
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[post("/api/v1/customer/check/annex/username")]
pub async fn check_username(
    customer: web::Json<AnnexIdCheckUsername>,
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
    if b.0 {
        let coll: Collection<Users> = client
            .database(&env::var("DATABASE_NAME").unwrap())
            .collection("annex_inc_users");

        if customer.u_name.is_none() {
            return HttpResponse::BadRequest().json(doc! {
                "status": false
            });
        }
        let result: Option<Users> = coll
            .find_one(doc! {
                "username": customer.u_name.clone().unwrap(),
            })
            .projection(doc! {
                "username" : 1,
                "gstin" : 1,
                "_id" : 1
            })
            .await
            .unwrap();

        match result {
            Some(user) => HttpResponse::Ok().json(AnnexResponse {
                _id: user._id,
                username: user.username.expect("Username not found"),
                gstin: user.gstin,
                msg: "User found".to_string(),
            }),
            None => HttpResponse::BadRequest().json(doc! {
                "status": false
            }),
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[post("/api/v1/customer/check/annex/gstin")]
pub async fn check_gstin(
    customer: web::Json<AnnexIdCheckGstin>,
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
    if b.0 {
        let coll: Collection<Users> = client
            .database(&env::var("DATABASE_NAME").unwrap())
            .collection("annex_inc_users");

        if customer.gstin.is_none() {
            return HttpResponse::BadRequest().finish();
        }
        let result: Option<Users> = coll
            .find_one(doc! {
                "gstin": customer.gstin.clone().unwrap(),
            })
            .projection(doc! {
                "username" : 1,
                "gstin" : 1,
                "_id" : 1
            })
            .await
            .unwrap();

        match result {
            Some(user) => HttpResponse::Ok().json(AnnexResponse {
                _id: user._id,
                username: user.username.expect("Username not found"),
                gstin: user.gstin,
                msg: "User found".to_string(),
            }),
            None => HttpResponse::BadRequest().json(doc! {
                "status": false
            }),
        }
    } else {
        HttpResponse::Unauthorized().finish()
    }
}


#[get("/api/v1/customers")]
pub async fn get_all_customers(
    client: web::Data<Client>, 
    req: HttpRequest) -> impl Responder {
    let token: &str = req.headers().get(header::AUTHORIZATION).and_then(
            |value| value.to_str().ok())
            .and_then(
                |value| value.strip_prefix("Bearer "))
            .unwrap_or("")
            .trim();
    let b: (bool, String) = validate_token(token, &client).await;
    if b.0 {
        let coll: Collection<Customer> = client
            .database(&env::var("DATABASE_NAME").unwrap())
            .collection("annex_inc_customers");

        let cursor: Vec<Customer> = coll.find(
            doc! {
                "u_id": &b.1
            },
        ).projection(doc! {
            "cre_at":0,
            "upd_at":0,
            "cls_bal":0,
            "cls_fine":0,
            "u_id":0
        }).limit(10).await.unwrap().try_collect::<Vec<_>>().await.unwrap();
        HttpResponse::Ok().json(cursor)
        
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[post("/api/v1/customers/estimate")]
pub async fn get_all_customers_estimate(
    data: web::Json<GetInvoices>,
    client: web::Data<Client>, 
    req: HttpRequest) -> impl Responder {
    let token: &str = req.headers().get(header::AUTHORIZATION).and_then(
            |value| value.to_str().ok())
            .and_then(
                |value| value.strip_prefix("Bearer "))
            .unwrap_or("")
            .trim();
    let b: (bool, String) = validate_token(token, &client).await;
    
    if b.0 {
        let u_coll: Collection<User> = client.database(&env::var("DATABASE_NAME").unwrap()).collection("annex_inc_users");

        let count: u64 = u_coll.count_documents(doc! {
            "_id": &data.anx_id
        }).await.unwrap();

        if count == 0 {
            return HttpResponse::BadRequest().json(doc! {
                "status": false
            });
        }        

        let coll: Collection<Customer> = client
            .database(&env::var("DATABASE_NAME").unwrap())
            .collection("annex_inc_estimate_invoice");

        let cursor: Vec<bson::Document> = coll.aggregate([
            doc! {
                "$match": {
                    "u_id": &data.anx_id
                }
            },
            doc! {
                "$limit": 5
            },
            doc! {
                "$lookup": {
                    "from": "annex_inc_customers",
                    "localField": "cst_name",
                    "foreignField": "_id",
                    "as": "customers",
                    "pipeline": [
                        {
                            "$match": {
                                "anx_id": &b.1
                            }
                        }
                    ]
                }
            },
            doc! {
                "$unwind": "$customers"
            },
            doc! {
                "$project": {
                   "_id": 1,
                   "t":1,
                   "inv_num":1,
                   "pymt_type":1,
                   "date":1,
                   "f":1,
                   "amt":1,
                }
            },
            doc! {
                "$sort": {
                    "cre_at": -1
                }
            }
        ]).await.unwrap().try_collect::<Vec<_>>().await.unwrap();
        HttpResponse::Ok().json(cursor)
        
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

