use crate::collections::{Comment, Session};
use crate::{AppState, APP_NAME};
use crate::{User, SESSION_LIFE};
use actix_web::cookie::time::{Duration, OffsetDateTime};
use actix_web::{delete, get, http, post, web, HttpRequest, HttpResponse, Responder, Result};
use chrono::prelude::*;
use futures::{StreamExt, TryStreamExt};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use qstring::QString;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[get("/comments")]
async fn get_comments(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let query_str = req.query_string();
    let qs = QString::from(query_str);

    if let Some(target_id) = qs.get("target_id") {
        let comment_collection = state
            .db
            .database(APP_NAME)
            .collection::<Comment>("comments");

        if let Ok(mut cursor) = comment_collection
            .find(doc! {"target_id": target_id}, None)
            .await
        {
            let mut comments = vec![];
            while let Some(Ok(doc)) = cursor.next().await {
                comments.push(doc);
            }
            return HttpResponse::Ok().json(comments);
        }
    }

    HttpResponse::BadRequest().finish()
}

#[post("/comments")]
async fn create_comment(
    req: HttpRequest,
    info: web::Json<Comment>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Some(session_id) = req.headers().get(http::header::AUTHORIZATION) {
        if let Ok(parseed_id) = session_id.to_str() {
            if let Ok(object_id) = ObjectId::parse_str(parseed_id) {
                let session_collection = state
                    .db
                    .database(APP_NAME)
                    .collection::<Session>("sessions");
                if let Ok(Some(_)) = session_collection
                    .find_one(doc! {"_id": object_id}, None)
                    .await
                {
                    // if let Ok(author_id) = ObjectId::parse_str(info.author.clone()) {

                    // }
                    let comment_collection = state
                        .db
                        .database(APP_NAME)
                        .collection::<Comment>("comments");
                    if let Ok(_) = comment_collection
                        .insert_one(
                            Comment {
                                target_id: info.target_id.clone(),
                                author: info.author.clone(),
                                likes: info.likes,
                                content: info.content.clone(),
                                id: None,
                            },
                            None,
                        )
                        .await
                    {
                        return HttpResponse::Ok().finish();
                    }
                }
            }
        }
    }

    HttpResponse::Unauthorized().finish()
}

#[delete("/comments")]
async fn delete_comment(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {

    let mut is_authenticated = false;

    if let Some(session_id) = req.headers().get(http::header::AUTHORIZATION) {
        if let Ok(parseed_id) = session_id.to_str() {
            if let Ok(object_id) = ObjectId::parse_str(parseed_id) {
                let session_collection = state.db.database(APP_NAME).collection::<Session>("sessions");
                if let Ok(Some(session)) = session_collection.find_one(doc! {"_id": object_id}, None).await {
                    // let user
                    is_authenticated = true;
                }
            }
        }
    }

    if !is_authenticated {
        return HttpResponse::Unauthorized().finish()
    }

    let query_str = req.query_string();
    let qs = QString::from(query_str);
    if let Some(comment_id) = qs.get("comment_id") {
        if let Ok(object_id) = ObjectId::parse_str(comment_id) {
            let comment_collection = state
                .db
                .database(APP_NAME)
                .collection::<Comment>("comments");
            if let Ok(Some(_)) = comment_collection
                .find_one_and_delete(doc! {"_id": object_id}, None)
                .await
            {
                return HttpResponse::Ok().finish();
            }
        }
    }
    HttpResponse::Unauthorized().finish()
}