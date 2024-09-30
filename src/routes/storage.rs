use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use bson::{doc, Document};
use futures::TryStreamExt;
use mongodb::{Client, Collection};
use std::env;

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

    match NamedFile::open_async(
        format!(
            "{}/{}",
            "/root/Annex/media/storage",
            v.get("name").unwrap().as_str().unwrap()
        )
    )
    .await {
        Ok(file) => file.into_response(&req),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/info/storage")]
pub async fn get_info(client: web::Data<Client>) -> impl Responder {
    let data: Collection<Document> = client
        .database(&env::var("CLOUD_DATABASE_NAME").unwrap())
        .collection("annex_inc_storage");

    let mut cursor = data.aggregate(
        vec![
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