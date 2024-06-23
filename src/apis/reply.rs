use crate::collections::{Reply, Session};
use crate::{AppState, APP_NAME, User};
use actix_web::{delete, get, http, post, put, web, HttpRequest, HttpResponse, Responder};
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use qstring::QString;

#[get("/replies")]
async fn get_replies(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let query_str = req.query_string();
    let qs = QString::from(query_str);

    if let Some(comment_id) = qs.get("comment_id") {
        if let Ok(object_id) = ObjectId::parse_str(comment_id) {
            let reply_collection = state.db.database(APP_NAME).collection::<Reply>("replies");
            if let Ok(mut cursor) = reply_collection
                .find(doc! {"comment_id": object_id}, None)
                .await
            {
                let mut replies = vec![];
                while let Some(Ok(doc)) = cursor.next().await {
                    replies.push(doc);
                }
                return HttpResponse::Ok().json(replies);
            }
        }
    }

    HttpResponse::BadRequest().finish()
}

#[post("replies")]
async fn create_reply(
    req: HttpRequest,
    info: web::Json<Reply>,
    state: web::Data<AppState>,
) -> impl Responder {
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
                    let reply_collection =
                        state.db.database(APP_NAME).collection::<Reply>("replies");

                    if let Ok(_) = reply_collection
                        .insert_one(
                            Reply {
                                comment_id: info.comment_id.clone(),
                                id: None,
                                content: info.content.clone(),
                                author: session.user_id.clone(),
                                likes: 0,
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

#[delete("/replies")]
async fn delete_reply(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
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

    let query_str = req.query_string();
    let qs = QString::from(query_str);

    if let Some(reply_id) = qs.get("reply_id") {
        if let Ok(object_id) = ObjectId::parse_str(reply_id) {
            let reply_collection = state.db.database(APP_NAME).collection::<Reply>("replies");
            if let Ok(Some(_)) = reply_collection
                .find_one_and_delete(doc! {"_id": object_id, "author": user_id}, None)
                .await
            {
                return HttpResponse::Ok().finish();
            }


            let user_collection = state.db.database(APP_NAME).collection::<User>("users");
            if let Ok(Some(_)) = user_collection.find_one(doc!{"_id": user_id, "role": "admin".to_string()}, None).await {
                if let Ok(Some(_)) = reply_collection.find_one_and_delete(doc!{"_id": object_id}, None).await {
                    return HttpResponse::Ok().finish(); 
                }
            }
        }
    }

    HttpResponse::Unauthorized().finish()
}

#[put("/replies")]
async fn update_reply(
    req: HttpRequest,
    info: web::Json<Reply>,
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

    if let Some(reply_id) = qs.get("reply_id") {
        if let Ok(object_id) = ObjectId::parse_str(reply_id) {
            let reply_collection = state.db.database(APP_NAME).collection::<Reply>("replies");
            if let Ok(Some(res)) = reply_collection
                .find_one_and_update(
                    doc! {"_id": object_id, "author": user_id},
                    doc! {"$set": doc!{"content": info.content.clone()}},
                    None,
                )
                .await
            {
                return HttpResponse::Ok().finish()
            }
        }
    }

    HttpResponse::Unauthorized().finish()
}
