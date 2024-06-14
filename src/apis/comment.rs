use crate::collections::Session;
use crate::{AppState, APP_NAME};
use crate::{User, SESSION_LIFE};
use actix_web::cookie::time::{Duration, OffsetDateTime};
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder, Result};
use chrono::prelude::*;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use qstring::QString;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[get("/comments/artist")]
async fn get_artist_comment(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let query_str = req.query_string();
    let qs = QString::from(query_str);

    if let Some(artist_id) = qs.get("artist_id") {
        
    }

    HttpResponse::BadRequest().finish()
}