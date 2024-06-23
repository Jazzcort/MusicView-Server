use crate::collections::{Comment, Session};
use crate::{AppState, APP_NAME, User};
use actix_web::{delete, get, http, post, put, web, HttpRequest, HttpResponse, Responder};
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use qstring::QString;

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

#[get("/comments/{comment_id}")]
async fn find_comment_by_id(req:HttpRequest, state: web::Data<AppState>) -> impl Responder {
    if let Some(comment_id) = req.match_info().get("comment_id") {
        if let Ok(object_id) = ObjectId::parse_str(comment_id) {
            let commend_collection = state.db.database(APP_NAME).collection::<Comment>("comments");

            if let Ok(Some(comment)) = commend_collection.find_one(doc! {"_id": object_id}, None).await {
                return HttpResponse::Ok().json(comment);
            }
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
    if let Some(comment_id) = qs.get("comment_id") {
        if let Ok(object_id) = ObjectId::parse_str(comment_id) {
            let comment_collection = state
                .db
                .database(APP_NAME)
                .collection::<Comment>("comments");
            if let Ok(Some(_)) = comment_collection
                .find_one_and_delete(doc! {"_id": object_id, "author": user_id}, None)
                .await
            {
                return HttpResponse::Ok().finish();
            }

            let user_collection = state.db.database(APP_NAME).collection::<User>("users");
            if let Ok(Some(_)) = user_collection.find_one(doc!{"_id": user_id, "role": "admin".to_string()}, None).await {
                if let Ok(Some(_)) = comment_collection.find_one_and_delete(doc!{"_id": object_id}, None).await {
                    return HttpResponse::Ok().finish(); 
                }
            }
        }
    }
    HttpResponse::Unauthorized().finish()
}

#[put("/comments")]
async fn update_comment(
    req: HttpRequest,
    info: web::Json<Comment>,
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

    let query_str = req.query_string();
    let qs = QString::from(query_str);
    if let Some(comment_id) = qs.get("comment_id") {
        if let Ok(object_id) = ObjectId::parse_str(comment_id) {
            let comment_collection = state
                .db
                .database(APP_NAME)
                .collection::<Comment>("comments");
            if let Ok(Some(_)) = comment_collection
                .find_one_and_update(doc! {"_id": object_id, "author": user_id}, doc! {"$set": doc!{ "content": info.content.clone() }}, None)
                .await
            {
                return HttpResponse::Ok().finish();
            }
        }
    }


    HttpResponse::Unauthorized().finish()
}
