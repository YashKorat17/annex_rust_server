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