use crate::collections::{Comment, Like, Reply, Session};
use crate::{AppState, APP_NAME};
use crate::{User, SESSION_LIFE};
use actix_web::cookie::time::{Duration, OffsetDateTime};
use actix_web::{delete, get, http, post, put, web, HttpRequest, HttpResponse, Responder, Result};
use chrono::prelude::*;
use futures::{StreamExt, TryStreamExt};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use qstring::QString;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[get("/likes")]
async fn is_like(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let query_str = req.query_string();
    let qs = QString::from(query_str);
    dbg!(&req);

    if let (Some(user_id), Some(target_id)) = (qs.get("user_id"), qs.get("target_id")) {
        if let (Ok(user_object_id), Ok(target_object_id)) =
            (ObjectId::parse_str(user_id), ObjectId::parse_str(target_id))
        {
            dbg!(&user_object_id, &target_object_id);
            let like_collection = state.db.database(APP_NAME).collection::<Like>("likes");
            if let Ok(Some(_)) = like_collection
                .find_one(
                    doc! {"user_id": user_object_id, "target_id": target_object_id},
                    None,
                )
                .await
            {
                return HttpResponse::Ok().json(json!({
                    "like": true
                }));
            } else {
                return HttpResponse::Ok().json(json!({
                    "like": false
                }));
            }
        }
    }

    HttpResponse::BadRequest().finish()
}

#[post("/likes")]
async fn create_like(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
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
                    is_authenticated = true;
                    user_id = session.user_id;
                }
            }
        }
    }

    if !is_authenticated {
        return HttpResponse::Unauthorized().finish();
    }

    let query_str = req.query_string();
    let qs = QString::from(query_str);

    if let Some(target_id) = qs.get("target_id") {
        if let Ok(target_object_id) = ObjectId::parse_str(target_id) {
            let like_collection = state.db.database(APP_NAME).collection::<Like>("likes");
            if let Ok(res) = like_collection
                .insert_one(
                    Like {
                        id: None,
                        user_id,
                        target_id: target_object_id,
                    },
                    None,
                )
                .await
            {
                let comment_collection = state
                    .db
                    .database(APP_NAME)
                    .collection::<Comment>("comments");
                let reply_collection = state.db.database(APP_NAME).collection::<Reply>("replies");

                if let Ok(Some(comment)) = comment_collection
                    .find_one(doc! {"_id": target_object_id}, None)
                    .await
                {
                    let new_value = if comment.likes == 9_223_372_036_854_775_807 {
                        comment.likes
                    } else {
                        comment.likes + 1
                    };
                    let _ = comment_collection
                        .find_one_and_update(
                            doc! {"_id": comment.id},
                            doc! {"$set": doc! {"likes": new_value as i64}},
                            None,
                        )
                        .await;
                }

                if let Ok(Some(reply)) = reply_collection
                    .find_one(doc! {"_id": target_object_id}, None)
                    .await
                {
                    let new_value = if reply.likes == 9_223_372_036_854_775_807 {
                        reply.likes
                    } else {
                        reply.likes + 1
                    };
                    let _ = reply_collection
                        .find_one_and_update(
                            doc! {"_id": reply.id},
                            doc! {"$set": doc!{"likes": new_value as i64}},
                            None,
                        )
                        .await;
                }
                return HttpResponse::Ok().finish();
            } else {
                return HttpResponse::BadRequest().finish();
            }
        }
    }

    HttpResponse::Unauthorized().finish()
}

#[delete("/likes")]
async fn delete_like(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
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
                    is_authenticated = true;
                    user_id = session.user_id;
                }
            }
        }
    }

    if !is_authenticated {
        return HttpResponse::Unauthorized().finish();
    }

    let query_str = req.query_string();
    let qs = QString::from(query_str);

    if let Some(target_id) = qs.get("target_id") {
        if let Ok(target_object_id) = ObjectId::parse_str(target_id) {
            let like_collection = state.db.database(APP_NAME).collection::<Like>("likes");
            if let Ok(Some(res)) = like_collection
                .find_one_and_delete(
                    doc! {"user_id": user_id, "target_id": target_object_id},
                    None,
                )
                .await
            {
                let comment_collection = state
                    .db
                    .database(APP_NAME)
                    .collection::<Comment>("comments");
                let reply_collection = state.db.database(APP_NAME).collection::<Reply>("replies");

                if let Ok(Some(comment)) = comment_collection
                    .find_one(doc! {"_id": res.target_id}, None)
                    .await
                {
                    let new_value = match comment.likes.checked_sub(1) {
                        Some(v) => v,
                        None => 0,
                    };
                    let _ = comment_collection
                        .find_one_and_update(
                            doc! {"_id": comment.id},
                            doc! {"$set": doc! {"likes": new_value as i64}},
                            None,
                        )
                        .await;
                }

                if let Ok(Some(reply)) = reply_collection
                    .find_one(doc! {"_id": res.target_id}, None)
                    .await
                {
                    let new_value = match reply.likes.checked_sub(1) {
                        Some(v) => v,
                        None => 0,
                    };
                    let _ = reply_collection
                        .find_one_and_update(
                            doc! {"_id": reply.id},
                            doc! {"$set": doc!{"likes": new_value as i64}},
                            None,
                        )
                        .await;
                }

                return HttpResponse::Ok().finish();
            } else {
                return HttpResponse::BadRequest().finish();
            }
        }
    }

    HttpResponse::Unauthorized().finish()
}
