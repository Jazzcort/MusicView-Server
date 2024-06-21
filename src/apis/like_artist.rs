use crate::collections::{Comment, Like, LikeArtist, Session};
use crate::{AppState, APP_NAME};
use crate::{User, SESSION_LIFE};
use actix_web::cookie::time::{Duration, OffsetDateTime};
use actix_web::{delete, get, http, post, put, web, HttpRequest, HttpResponse, Responder, Result};
use chrono::prelude::*;
use futures::{Stream, StreamExt, TryStreamExt};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use qstring::QString;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::user;

#[post("/artists")]
async fn like_artist(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
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

    let user_collection = state.db.database(APP_NAME).collection::<User>("users");
    if let Ok(Some(_)) = user_collection
        .find_one(doc! {"_id": user_id, "role": "fan"}, None)
        .await
    {
        let query_str = req.query_string();
        let qs = QString::from(query_str);

        if let Some(artist_id) = qs.get("artist_id") {
            let like_artist_collection = state
                .db
                .database(APP_NAME)
                .collection::<LikeArtist>("like_artists");
            if let Ok(_) = like_artist_collection
                .insert_one(
                    LikeArtist {
                        id: None,
                        user_id,
                        artist_id: artist_id.to_string(),
                    },
                    None,
                )
                .await
            {
                return HttpResponse::Ok().finish();
            }
        }
    }

    HttpResponse::Unauthorized().finish()
}

#[delete("/artists")]
async fn dislike_artist(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
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

    if let Some(artist_id) = qs.get("artist_id") {
        let like_artist_collection = state
            .db
            .database(APP_NAME)
            .collection::<LikeArtist>("like_artists");
        if let Ok(Some(_)) = like_artist_collection
            .find_one_and_delete(doc! {"user_id": user_id, "artist_id": artist_id}, None)
            .await
        {
            return HttpResponse::Ok().finish();
        }
    }

    HttpResponse::Unauthorized().finish()
}

#[get("/artists")]
async fn is_like_artist(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let query_str = req.query_string();
    let qs = QString::from(query_str);

    if let (Some(user_id), Some(artist_id)) = (qs.get("user_id"), qs.get("artist_id")) {
        if let Ok(user_object_id) = ObjectId::parse_str(user_id) {
            let like_artist_collection = state
                .db
                .database(APP_NAME)
                .collection::<LikeArtist>("like_artists");
            if let Ok(Some(res)) = like_artist_collection
                .find_one(
                    doc! {"user_id": user_object_id, "artist_id": artist_id},
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

#[get("/artists/{user_id}")]
async fn get_liked_artists(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    if let Some(user_id) = req.match_info().get("user_id") {
        if let Ok(user_object_id) = ObjectId::parse_str(user_id) {
            let like_artist_collection = state
                .db
                .database(APP_NAME)
                .collection::<LikeArtist>("like_artists");
            if let Ok(mut cursor) = like_artist_collection
                .find(doc! {"user_id": user_object_id}, None)
                .await
            {

                let mut likes = vec![];
                while let Some(Ok(doc)) = cursor.next().await {
                    likes.push(doc.artist_id);

                }
                let mut response = vec![];
                while likes.len() != 0 && response.len() < 10 {
                    response.push(likes.pop().unwrap());
                }

                return HttpResponse::Ok().json(response);
            }
        }
    }

    HttpResponse::BadRequest().finish()
}
