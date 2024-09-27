#[path = "../models/auth.rs"]
pub mod auth;
#[path = "../models/estimate.rs"]
pub mod e_model;


use actix_web::{get, http::header, post, web, HttpRequest, HttpResponse, Responder};
use auth::validate_token;
use bson::{doc, Document};
use chrono::Datelike;
use e_model::{InvoiceIdReturn, Search};
use futures::TryStreamExt;
use mongodb::{Client, Collection};
use std::env;



#[get("/get/id/{month}/{year}")]
pub async fn get_inv_id(
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
        .collection("annex_inc_invoice");


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
pub async fn get_inv(
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
        .collection("annex_inc_invoice");

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

    let mut query:Vec<Document> = vec![
        doc! {
            "$match" :doc! {
                "is_del": false,
                "u_id": &b.1,
            }
        },
        doc! {
            "$lookup": {
                "from": "annex_inc_customers",
                "localField": "cst_name",
                "foreignField": "_id",
                "as": "customer"
            }
        },
        doc!{
            "$unwind": "$customer"
        }
    ];

    if let Some(_cst_name) = &data.cst_name {
        query.push(doc!{"$match": {
                    "$or": [
                        {
                            "customer.name": {
                                "$regex": &data.cst_name,
                                "$options": "i"
                            }
                        },
                        {
                            "customer.b_name": {
                                "$regex": &data.cst_name,
                                "$options": "i"
                            }
                        }
                    ]
                }});
    }

    if data.y != 0 {
        query.push(doc!{"$match":doc! {
            "date": doc!{
                "$gte": format!("{}-04-01", data.y),
                "$lt": format!("{}-03-31", data.y+1)
            }
        }});
    }

    query.push(doc!{"$sort": doc!{"date": -1}});
    query.push(doc!{"$project": doc!{
        "_id": 0,
        "id": "$_id",
        "cust_name": "$customer.name",
        "b_name": "$customer.business_name",
        "t": 1,
        "inv_num": 1,
        "date": 1,
        "pymt_type": 1,
        "amt": 1,
        "f": 1,
        "is_fin":1,
        "is_vrfy":1
    }});

    query.push(doc!{"$facet": {
        "data": [{ "$skip": (data.p.unwrap_or(1) - 1) * data.l.unwrap_or(10) }, { "$limit": data.l.unwrap_or(10) }],
        "metadata": [{ "$count": "total" }]
    }});


    let coll: Collection<Document> = client
        .database(&env::var("DATABASE_NAME").unwrap())
        .collection("annex_inc_invoice");

    let cursor: Vec<Document> = coll.aggregate(query).await.unwrap().try_collect::<Vec<_>>().await.unwrap();

    HttpResponse::Ok().json(cursor)
}