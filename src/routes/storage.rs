#[path = "../models/auth.rs"]
pub mod auth;

use actix_files::NamedFile;
use actix_web::{get, http::header::{self, ContentDisposition, DispositionParam, DispositionType}, web, HttpRequest, HttpResponse, Responder};
use auth::validate_token;
use bson::{doc, Document};
use futures::TryStreamExt;
use mongodb::{Client, Collection};
use std::{env, fs};

#[get("/{id}")]
pub async fn get_media(
    id: web::Path<String>,
    client: web::Data<Client>,
    req: HttpRequest,
) -> impl Responder {
    let data: Collection<Document> = client
        .database(&env::var("CLOUD_DATABASE_NAME").unwrap())
        .collection("annex_inc_storage");

    let v = data
        .find_one(doc! {"_id": id.to_string()})
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

    if v.get("o").unwrap().as_str().unwrap() == "P" {
        return HttpResponse::Ok().json(
            doc! {
                "msg": "You Are Not Allowed To Access This File",
            }
        );
    }

    let headers = ContentDisposition {
        disposition: DispositionType::Inline,
        parameters: vec![
        DispositionParam::Filename(
            format!(
                "{}",
                v.get("name").unwrap().as_str().unwrap()
            )
        ),
        ],
    };

    match NamedFile::open_async(
        format!(
            "{}/{}",
            "storage",
            v.get("name").unwrap().as_str().unwrap()
        )
    )
    .await {
        Ok(file) => {
            file.set_content_disposition(headers).into_response(&req)
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/temp/{id}")]
pub async fn get_temp_media(
    id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
   
    let headers = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![],
    };

    match NamedFile::open_async(
        format!(
            "{}/{}",
            "storage/temp",
            id.to_string()
        )
    )
    .await {
        Ok(file) => {
            fs::remove_file(format!(
                "{}/{}",
                "storage/temp",
                id.to_string()
            )).unwrap();
            file.set_content_disposition(headers).into_response(&req)
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}



#[get("/info/storage")]
pub async fn get_info(
    client: web::Data<Client>,
    req: HttpRequest
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

    let data: Collection<Document> = client
        .database(&env::var("CLOUD_DATABASE_NAME").unwrap())
        .collection("annex_inc_storage");

    let mut cursor = data.aggregate(
        vec![
            doc! {
                "$match": {
                    "u_id": &b.1
                }
            },
            doc! {
                "$group": {
                    "_id": "$o",
                    "count": {
                        "$sum": "$s"
                    }
                }
            },
        ],
    ).await.unwrap();

    let mut data = doc! {
        "public": 0,
        "private": 0,
        "total": 0,
        "available": 0,
    };

    while let Some(doc) = cursor.try_next().await.unwrap() {
        if doc.get("_id").unwrap().as_str().unwrap() == "P" {
            data.insert("public", doc.get("count").unwrap());
        } else {
            data.insert("private", doc.get("count").unwrap());
        }
    }

    data.insert("total",
        data.get("public").unwrap().as_i32().unwrap() + data.get("private").unwrap().as_i32().unwrap()
    );

    data.insert("available", 1073741824 - data.get("total").unwrap().as_i32().unwrap());


    HttpResponse::Ok().json(data)

}
