use crate::collections::{Session, UserUpdateForm};
use crate::{AppState, APP_NAME};
use crate::{User, SESSION_LIFE};
use actix_web::{get, post, put, web, http, HttpRequest, HttpResponse, Responder};
use chrono::prelude::*;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use qstring::QString;
use serde::{Deserialize, Serialize};
use serde_json::json;
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
async fn login(
    req: HttpRequest,
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

                // let mut cookie = Cookie::new("user_session", id.clone());
                // let mut expiration_time = OffsetDateTime::now_utc();
                // expiration_time += Duration::seconds(SESSION_LIFE);
                // cookie.set_domain(std::env::var("SERVER_DOMAIN").expect("Can't find server_domain"));
                // cookie.set_expires(expiration_time);
                // cookie.set_same_site(SameSite::None);
                // cookie.set_secure(true);
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
                role: Some("fan".to_string()),
                artist_id: None,
            },
            None,
        )
        .await
    {
        return HttpResponse::Ok().finish();
    }

    HttpResponse::BadRequest().finish()
}

#[get("users/user_info")]
async fn get_user(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let query_str = req.query_string();

    let qs = QString::from(query_str);
    if let Some(session_id) = qs.get("session_id") {
        let session_collection = state
            .db
            .database(APP_NAME)
            .collection::<Session>("sessions");

        if let Ok(object_id) = ObjectId::parse_str(session_id) {
            if let Ok(Some(session)) = session_collection
                .find_one(doc! { "_id": object_id }, None)
                .await
            {
                let user_collection = state.db.database(APP_NAME).collection::<User>("users");

                if let Ok(Some(user)) = user_collection
                    .find_one(doc! {"_id": session.user_id}, None)
                    .await
                {
                    let response = json!( {
                        "username": user.username,
                        "email": user.email,
                        "role": user.role.unwrap(),
                        "id": user.id.unwrap(),
                        "artist_id": user.artist_id,
                    });
                    return HttpResponse::Ok().json(response);
                }
            }
        }
    }

    HttpResponse::BadRequest().finish()
}

#[get("/users/search_user")]
async fn search_user(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let query_str = req.query_string();
    let qs = QString::from(query_str);
    if let Some(user_id) = qs.get("user_id") {
        if let Ok(object_id) = ObjectId::parse_str(user_id) {
            let user_collection = state.db.database(APP_NAME).collection::<User>("users");
            if let Ok(Some(user)) = user_collection
                .find_one(doc! {"_id": object_id}, None)
                .await
            {
                let res = json!({
                    "role": user.role,
                    "username": user.username,
                    "id": user.id,
                    "artist_id": user.artist_id,
                });
                return HttpResponse::Ok().json(res);
            }
        }
    }
    HttpResponse::BadRequest().finish()
}

#[put("/users")]
async fn update_user(
    req: HttpRequest,
    info: web::Json<UserUpdateForm>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mut is_authenticated = false;
    let mut user_id = ObjectId::new();

    if let Some(session_id) = req.headers().get(http::header::AUTHORIZATION) {
        if let Ok(parseed_id) = session_id.to_str() {
            if let Ok(object_id) = ObjectId::parse_str(parseed_id) {
                let session_collection = state
                    .db
                    .database(APP_NAME)
                    .collection::<Session>("sessions");
                if let Ok(Some(session)) = session_collection
                    .find_one(doc! {"_id": object_id}, None)
                    .await
                {
                    // let user
                    is_authenticated = true;
                    user_id = session.user_id;
                }
            }
        }
    }

    if !is_authenticated {
        return HttpResponse::Unauthorized().finish();
    }

    let user_collection = state.db.database(APP_NAME).collection::<User>("users");

    if let Some(email) = &info.email {
        if let Ok(Some(res)) = user_collection
            .find_one_and_update(
                doc! {"_id": user_id},
                doc! {"$set": {"email": email.to_string()}},
                None,
            )
            .await
        {
            return HttpResponse::Ok().finish();
        }
    } else if let Some(username) = &info.username {
        if let Ok(Some(res)) = user_collection
            .find_one_and_update(
                doc! {"_id": user_id},
                doc! {"$set": {"username": username.to_string()}},
                None,
            )
            .await
        {
            return HttpResponse::Ok().finish();
        }
    }

    HttpResponse::BadRequest().finish()
}
