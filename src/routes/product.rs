#[path = "../models/auth.rs"]
pub mod auth;
#[path = "../models/product.rs"]
mod product;

use std::env;

use actix_web::{get, http::header, post, web, HttpRequest, HttpResponse, Responder};
use auth::validate_token;
use bson::{doc, Document};
use mongodb::{Client, Collection};
use product::Search;
use futures::TryStreamExt;


#[get("/category")]
async fn get_category(client: web::Data<Client>, req: HttpRequest) -> impl Responder {
    let token: &str = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or("")
        .trim();
    let b: (bool, String) = validate_token(token, &client).await;

    if !b.0 {
        return HttpResponse::Unauthorized().finish();
    }

    let coll: Collection<Document> = client
        .database(&env::var("DATABASE_NAME").unwrap())
        .collection("annex_inc_category");

    let cursor: Vec<Document> = coll
        .aggregate(vec![
            doc! {
                "$match": {
                    "is_del": false,
                    "u_id": &b.1
                }
            },
            doc! {
                "$group": {
                    "_id": "$comm"
                }
            },
            doc! {
                "$project": {
                    "_id": 0,
                    "name": "$_id"
                }
            },
        ])
        .await
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    HttpResponse::Ok().json(cursor)
}

#[post("/search")]
async fn search_product(
    data: web::Json<Search>,
    client: web::Data<Client>,
    req: HttpRequest,
) -> impl Responder {
    let token: &str = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or("")
        .trim();
    let b: (bool, String) = validate_token(token, &client).await;

    if !b.0 {
        return HttpResponse::Unauthorized().finish();
    }

    let mut query = doc! {
        "is_del": false,
            "is_act": true,
            "is_del": false,
            "is_selb": true,
            "u_id": &b.1
    };

    if let Some(_name) = &data.name {
        query.insert("name", doc! { "$regex": _name, "$options": "i" });
    }

    if let Some(f) = &data.f {
        query.insert("itm.tch", doc! { "$gte": f });
    }

    if let Some(t) = &data.t {
        query.insert("itm.tch", doc! { "$lte": t });
    }

    if let Some(mark) = &data.mark {
        query.insert("mark", doc! { "$regex": mark, "$options": "i" });
    }

    let coll: Collection<Document> = client
        .database(&env::var("DATABASE_NAME").unwrap())
        .collection("annex_inc_products");

    let cursor: Vec<Document> = coll.aggregate(vec![
        doc! {
            "$match": query
        },
        doc! {
            "$unwind": doc! {
                "path": "$itm",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$unwind": doc! {
                "path": "$itm.tch",
                "preserveNullAndEmptyArrays": true
            }

        },
        doc! {
            "$sort": doc! {
                "tch": -1,
                "op_s": -1
            }
        },
        doc! {
            "$project": {
                "_id":0,
                "count": "$total",
                    "id":"$_id",
                    "name":1,
                    "mark":1,
                    "tch":"$itm.tch",
                    "op_s":"$itm.wgt",
                    "sl_w":1,
                    "prs_w":1,
                    "sl_r":1,
                    "prs_r":1,
                    "gst":1,
                    "hsn":1,
                    "tx":1,
            }
        },
        doc! {
            "$facet": {
                "data": [{ "$skip": (data.p.unwrap_or(1) - 1) * data.l.unwrap_or(10) }, { "$limit": data.l.unwrap_or(10) }],
                "metadata": [{ "$count": "total" }]
            }
        }
    ]).await.unwrap().try_collect::<Vec<_>>().await.unwrap(); 


    HttpResponse::Ok().json(cursor)
}
