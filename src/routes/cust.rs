#[path = "../models/auth.rs"]
pub mod auth;
#[path = "../models/cust.rs"]
pub mod cust_model;

use actix_web::{get, http::header, post, web, HttpRequest, HttpResponse, Responder};
use auth::{validate_token, User};
use bson::{doc, Document};
use cust_model::{AnnexIdCheckGstin, AnnexIdCheckUsername, AnnexResponse, Customer, GetInvoices, Search, Users , StatementId};
use futures::TryStreamExt;
use mongodb::{Client, Collection};
use std::env;


#[get("/test")]
pub async fn test() -> impl Responder {
    println!("Test");
    HttpResponse::Ok().json(doc! {
        "status": true,
        "msg": "Test"
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
            _none => HttpResponse::BadRequest().json(doc! {
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

#[post("/api/v1/customer/estimate")]
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

#[post("/api/v1/customer/payments")]
pub async fn get_all_customers_payments(
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
            .collection("annex_inc_payment");

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
                   "pyt_num":1,
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

#[post("/api/v1/customer/search")]
pub async fn search_customers(
    data: web::Json<Search>,
    client: web::Data<Client>, 
    req: HttpRequest) -> impl Responder {
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

    let mut query: Document  = doc! {
        "is_act": true,
        "is_del": false,
        "u_id": &b.1
    };

    if let Some(_cst_name) = &data.cst_name {
        query.insert("name", doc!{
            "$regex": _cst_name,
            "$options": "i"
        });
    }

    if let Some(_city) = &data.city {
        query.insert("city", _city);
    }

    if let Some(_state) = &data.state {
        query.insert("state", _state);
    }


    let coll: Collection<Customer> = client
        .database(&env::var("DATABASE_NAME").unwrap())
        .collection("annex_inc_customers");

    let cursor: Vec<Document> = coll.aggregate([
        doc! {
            "$match": query
        },
        doc!{"$sort": doc!{"cls_fine": 1}},
        doc! {
            "$project": {
                "_id": 0,
                "id":"$_id",
                "name": 1,
                "b_name": 1,
                "city": 1,
                "state": 1,
                "gstin": 1,
                "ph": {
                    "$first": "$ph"
                }
            }
        },
        doc! {
            "$facet": {
                "metadata": [{ "$count": "total" }],
                "data": [
                    { "$skip": (data.p.unwrap_or(1) - 1) * data.l.unwrap_or(10) },
                    { "$limit": data.l.unwrap_or(10) }
                ]
            }
        }
    ]).await.unwrap().try_collect::<Vec<_>>().await.unwrap();

    HttpResponse::Ok().json(cursor)

}

#[post("/api/v1/customer/statement")]
pub async fn get_customer_statement(
    data: web::Json<StatementId>,
    client: web::Data<Client>, 
    req: HttpRequest) -> impl Responder {
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

    let coll: Collection<Customer> = client
        .database(&env::var("DATABASE_NAME").unwrap())
        .collection("annex_inc_customers");

    let cursor: Vec<Document> = coll.aggregate([
        doc! {
            "$match": {
                "is_act": true,
                "is_del": false,
                "u_id": &b.1,
                "_id": &data.id,
            }
        },
        doc! {
            "$lookup": {
                "from": "annex_inc_estimate_invoice",
                "localField": "_id",
                "foreignField": "cst_name",
                "as": "estimate",
                "pipeline": [
                    {
                        "$match": {
                            "is_del": false,
                            "u_id": &b.1
                        }
                    },
                    {
                        "$lookup": {
                            "from": "annex_inc_estimate_invoice",
                            "localField": "anx_est_id",
                            "foreignField": "_id",
                            "as": "anx_est"
                        }
                    },
                    {
                        "$unwind": {
                            "path": "$anx_est",
                            "preserveNullAndEmptyArrays": true
                        }
                    }         
                    ,{
                        "$project": {
                            "_id": 1,
                            "t": 1,
                            "inv_num": 1,
                            "pymt_type": 1,
                            "date": 1,
                            "f": 1,
                            "is_vrfy": 1,
                            "amt": 1,
                            "anx_est": {
                                "t": 1,
                                "pymt_type": 1,
                                "date": 1,
                                "f": 1,
                                "amt": 1,
                                "is_vrfy": 1,
                            }
                        }
                    },
                    {
                        "$sort": {
                            "inv_num": 1
                        }
                    }
                ]
            }
        },

        doc! {
            "$lookup": {
                "from": "annex_inc_payment",
                "localField": "_id",
                "foreignField": "cst_name",
                "as": "payment",
                "pipeline": [
                    {
                        "$match": {
                            "is_del": false,
                            "u_id": &b.1
                        }
                    },
                    {
                        "$lookup": {
                            "from": "annex_inc_payment",
                            "localField": "anx_pyt_id",
                            "foreignField": "_id",
                            "as": "anx_pyt"
                        }
                    },
                    {
                        "$unwind": {
                            "path": "$anx_pyt",
                            "preserveNullAndEmptyArrays": true
                        }
                    },
                    {
                        "$project": {
                            "_id": 1,
                            "t": 1,
                            "pyt_num": 1,
                            "date": 1,
                            "f": 1,
                            "amt": 1,
                            "is_vrfy": 1,
                            "anx_pyt": {
                                "t": 1,
                                "date": 1,
                                "f": 1,
                                "amt": 1,
                                "is_vrfy": 1
                            }
                        }
                    },
                    {
                        "$sort": {
                            "pyt_num": 1
                        }
                    }
                ]
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "id": "$_id",
                "name": 1,
                "b_name": 1,
                "city": 1,
                "state": 1,
                "gstin": 1,
                "ph": 1,
                "op_bal": 1,
                "op_fine": 1,
                "cls_bal": 1,
                "cls_fine": 1,
                "estimate": 1,
                "payment": 1
            }
        }
    ]).await.unwrap().try_collect::<Vec<_>>().await.unwrap();

    HttpResponse::Ok().json(cursor)

}
