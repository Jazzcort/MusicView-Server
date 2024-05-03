use crate::collections::Session;
use crate::{AppState, APP_NAME};
use crate::{User, SESSION_LIFE};
use actix_web::cookie::time::{Duration, OffsetDateTime};
use actix_web::cookie::Cookie;
use actix_web::error::ErrorConflict;
use actix_web::{get, post, web, delete, HttpRequest, HttpResponse, Responder, Result};
use chrono::prelude::*;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct LonginInfo {
    email: String,
    password: String,
}

#[derive(Deserialize, Serialize)]
struct LoginSession {
    session_id: String,
    expiration_date: i64,
}

#[derive(Deserialize)]
struct SessionId {
    session_id: String
}

#[get("/user/emailCheck/{email}")]
async fn email_exists(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let email: String = req.match_info().get("email").unwrap().parse().unwrap();

    let collection = state.db.database(APP_NAME).collection::<User>("users");

    match collection.find_one(doc! { "email": email }, None).await {
        Ok(res) => {
            if let Some(user) = res {
                dbg!(&user);
                return HttpResponse::Ok().body("1");
            }

            return HttpResponse::Ok().body("0");
        }
        Err(_) => return HttpResponse::NoContent().finish(),
    };
}

#[get("/user/usernameCheck/{username}")]
async fn username_exists(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let username: String = req.match_info().get("username").unwrap().parse().unwrap();

    let collection = state.db.database(APP_NAME).collection::<User>("users");

    match collection
        .find_one(doc! { "username": username }, None)
        .await
    {
        Ok(res) => {
            if let Some(user) = res {
                dbg!(&user);
                return HttpResponse::Ok().body("1");
            }

            return HttpResponse::Ok().body("0");
        }
        Err(_) => return HttpResponse::NoContent().finish(),
    };
}

#[post("/user/create")]
async fn create_user(info: web::Json<User>, state: web::Data<AppState>) -> impl Responder {
    let collection = state.db.database(APP_NAME).collection::<User>("users");

    match collection
        .insert_one(
            User {
                id: None,
                email: info.email.to_string().to_lowercase(),
                username: info.username.to_string(),
                hash: info.hash.to_string(),
                salt: info.salt.to_string(),
            },
            None,
        )
        .await
    {
        Ok(_) => Ok("success".to_string()),
        Err(e) => Err(ErrorConflict(e)),
    }
}

#[delete("/user/delete")]
async fn delete_user(info: web::Json<SessionId>, state: web::Data<AppState>) -> impl Responder {
    let session_collection = state.db.database(APP_NAME).collection::<Session>("sessions");

    match ObjectId::parse_str(&info.session_id) {
        Ok(object_id) => {
            let _ = session_collection.delete_one(doc!{ "_id": object_id  }, None).await;
            HttpResponse::Ok()
        }
        Err(_) => {
            HttpResponse::BadRequest()
        }
    }

}

#[post("/user/loginPassword")]
async fn login_with_password(
    info: web::Json<LonginInfo>,
    state: web::Data<AppState>,
) -> impl Responder {
    let collection = state.db.database(APP_NAME).collection::<User>("users");

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

                // let mut cookie = Cookie::new("user_session", id);
                // let mut expiration_time = OffsetDateTime::now_utc();
                // expiration_time += Duration::seconds(SESSION_LIFE);
                // cookie.set_expires(expiration_time);
                // cookie.set_path("/");
                // cookie.set_http_only(true);
                let session_json = LoginSession {
                    session_id: id,
                    expiration_date: cur.timestamp() + SESSION_LIFE,
                };

                return HttpResponse::Ok().json(session_json);
            }

            return HttpResponse::InternalServerError().finish();
        } else {
            return HttpResponse::Forbidden().finish();
        }
    }

    HttpResponse::Unauthorized().finish()
}

#[post("/user/loginSession")]
async fn login_with_session(
    info: web::Json<LoginSession>,
    state: web::Data<AppState>,
) -> impl Responder {
    let session_collection = state
        .db
        .database(APP_NAME)
        .collection::<Session>("sessions");

    // if let Some(cookie) = req.cookie("user_session") {
    //     let session_id = cookie.value();
    //     if let Ok(Some(session)) = session_collection
    //         .find_one(
    //             doc! { "_id": ObjectId::parse_str(session_id).unwrap() },
    //             None,
    //         )
    //         .await
    //     {
    //         if Utc::now().timestamp() <= session.expiration_date {
    //             return HttpResponse::Ok().finish();
    //         }
    //     }
    // }

    let object_id = match ObjectId::parse_str(&info.session_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().finish();
        }
    };

    if let Ok(Some(session)) = session_collection
        .find_one(
            doc! { "_id": object_id },
            None,
        )
        .await
    {
        if Utc::now().timestamp() <= session.expiration_date {
            return HttpResponse::Ok().finish();
        }
    }

    HttpResponse::Unauthorized().finish()
}

#[post("/testing")]
async fn index(
    req: HttpRequest,
    info: web::Json<User>,
    state: web::Data<AppState>,
) -> Result<String> {
    let cookie = req.cookie("user_session").unwrap();

    let session_collection = state
        .db
        .database(APP_NAME)
        .collection::<Session>("sessions");
    let a = session_collection
        .find_one(
            doc! { "_id": ObjectId::parse_str(cookie.value()).unwrap() },
            None,
        )
        .await;
    dbg!(a);
    Ok(format!("Welcome {}!", info.username))
}
