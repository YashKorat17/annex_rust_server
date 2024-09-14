use bson::doc;
use jsonwebtoken::{decode, DecodingKey, Validation};
use mongodb::{Client, Collection};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub username: String, // Optional. Subject (whom token refers to)
}
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub _id: String,
}

pub async fn validate_token(token: &str, client: &Client) -> (bool, String) {
    
    if token.is_empty() {
        return (false, String::from("Token is empty"));
    }

    let token_data: Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> =
        decode::<Claims>(
            &token,
            &DecodingKey::from_secret(env::var("SECRET_KEY").unwrap().as_ref()),
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        );

    match token_data {
        Ok(_) => {
            let user_collection: Collection<User> = client
                .database(&env::var("DATABASE_NAME").unwrap())
                .collection("annex_inc_users");
            let user_doc: Option<User> = user_collection
                .find_one(doc! {
                    "username": token_data.unwrap().claims.username
                })
                .await
                .unwrap();
            match user_doc {
                Some(user) => (true, user._id),
                None => (false, String::from("User not found")),
            }
        }
        Err(_) => (false, String::from("Token is invalid")),
    }
}

// pub async  fn validatorMiddleware(
//     req:ServiceRequest,
//     credentials: actix_web_httpauth::extractors::bearer::BearerAuth,
// ) -> Result<ServiceRequest, actix_web::Error> {
//     let client: Client = req.app_data::<Client>().unwrap().clone();
//     let token: &str = credentials.token();
//     let b: (bool, String) = validate_token(token, &client).await;
//        if b.0 {
//         Ok(req)
//     } else {
//         Err(actix_web::error::ErrorUnauthorized("Unauthorized"))
//     }
// }