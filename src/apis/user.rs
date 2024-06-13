use crate::collections::Session;
use crate::{AppState, APP_NAME};
use crate::{User, SESSION_LIFE};
use actix_web::cookie::time::{Duration, OffsetDateTime};
use actix_web::cookie::Cookie;
use actix_web::error::ErrorConflict;
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder, Result};
use chrono::prelude::*;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use qstring::QString;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Deserialize, Debug)]
struct LonginInfo {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginSession {
    session_id: String,
    expiration_date: i64,
}

#[post("/users/login")]
async fn login(info: web::Json<LonginInfo>, state: web::Data<AppState>) -> impl Responder {
    let collection = state.db.database(APP_NAME).collection::<User>("users");
    dbg!(&info);
    // let mut hasher2 = Sha256::new();
    // hasher2.update(&info.password);
    // let password_hash = format!("{:x}", hasher2.finalize());
    // dbg!(password_hash);

    if let Ok(Some(res)) = collection
        .find_one(doc! { "email": info.email.to_string() }, None)
        .await
    {
        let mut hasher = Sha256::new();
        let salted_password = info.password.to_string() + &res.salt;
        hasher.update(salted_password);
        let hash_res = hasher.finalize();
        let tmp_str: String = format!("{:x}", hash_res);

        if tmp_str == res.hash {
            let session_collection = state
                .db
                .database(APP_NAME)
                .collection::<Session>("sessions");
            let cur = Utc::now();
            if let Ok(object_id) = session_collection
                .insert_one(
                    Session {
                        user_id: res.id.clone().unwrap(),
                        expiration_date: cur.timestamp() + SESSION_LIFE,
                    },
                    None,
                )
                .await
            {
                let id = object_id.inserted_id.as_object_id().unwrap().to_hex();

                // let mut cookie = Cookie::new("user_session", id.clone());
                // let mut expiration_time = OffsetDateTime::now_utc();
                // expiration_time += Duration::seconds(SESSION_LIFE);
                // cookie.set_domain("localhost");
                // cookie.set_expires(expiration_time);
                // cookie.set_path("/");
                // cookie.set_http_only(true);
                let session_json = LoginSession {
                    session_id: id,
                    expiration_date: cur.timestamp() + SESSION_LIFE,
                };

                return HttpResponse::Ok().json(session_json);
            }

            return HttpResponse::Unauthorized().finish();
        }
    }

    HttpResponse::Unauthorized().finish()
}

#[post("/users/register")]
async fn register(info: web::Json<User>, state: web::Data<AppState>) -> impl Responder {
    let user_collection = state.db.database(APP_NAME).collection::<User>("users");
    dbg!(&info);

    let mut hasher = Sha256::new();
    let salted_password = info.hash.to_string() + &info.salt;
    hasher.update(salted_password);
    let hash_res = format!("{:x}", hasher.finalize());

    if let Ok(_) = user_collection
        .insert_one(
            User {
                id: None,
                email: info.email.to_string().to_lowercase(),
                username: info.username.to_string(),
                hash: hash_res,
                salt: info.salt.to_string(),
            },
            None,
        )
        .await
    {
        return HttpResponse::Ok().finish();
    }

    HttpResponse::BadRequest().finish()
}

#[get("users/username")]
async fn get_username(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let query_str = req.query_string();
    let qs = QString::from(query_str);
    if let Some(session_id) = qs.get("session_id") {
        let session_collection = state
            .db
            .database(APP_NAME)
            .collection::<Session>("sessions");

        if let Ok(object_id) = ObjectId::parse_str(session_id) {
            if let Ok(Some(session)) = session_collection.find_one(doc! { "_id": object_id }, None).await {
                let user_collection = state.db.database(APP_NAME).collection::<User>("users");

                if let Ok(Some(user)) = user_collection.find_one(doc! {"_id": session.user_id}, None).await {
                    return HttpResponse::Ok().json(user);
                }
            }
            // dbg!(res);
        }
    }

    HttpResponse::BadRequest().finish()
}
