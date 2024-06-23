use crate::collections::{Comment, Like, Reply, Session};
use crate::{AppState, APP_NAME};
use actix_web::{delete, get, http, post, web, HttpRequest, HttpResponse, Responder};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use qstring::QString;
use serde_json::json;

#[get("/likes")]
async fn is_like(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let query_str = req.query_string();
    let qs = QString::from(query_str);

    if let (Some(user_id), Some(target_id)) = (qs.get("user_id"), qs.get("target_id")) {
        if let (Ok(user_object_id), Ok(target_object_id)) =
            (ObjectId::parse_str(user_id), ObjectId::parse_str(target_id))
        {
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

    if let (Some(target_id), Some(target)) = (qs.get("target_id"), qs.get("target")) {
        if let Ok(target_object_id) = ObjectId::parse_str(target_id) {
            match target {
                "comment" | "reply" => {
                    let like_collection = state.db.database(APP_NAME).collection::<Like>("likes");
                    if let Ok(_) = like_collection
                        .insert_one(
                            Like {
                                id: None,
                                user_id,
                                target_id: target_object_id,
                                target: target.to_string(),
                            },
                            None,
                        )
                        .await
                    {
                        match target {
                            "comment" => {
                                let comment_collection = state
                                    .db
                                    .database(APP_NAME)
                                    .collection::<Comment>("comments");
                                let _ =
                                    increase_likes_comment(comment_collection, target_object_id)
                                        .await;
                            }
                            "reply" => {
                                let reply_collection =
                                    state.db.database(APP_NAME).collection::<Reply>("replies");
                                let _ =
                                    increase_likes_reply(reply_collection, target_object_id).await;
                            }
                            _ => {}
                        }

                        // let comment_collection = state
                        //     .db
                        //     .database(APP_NAME)
                        //     .collection::<Comment>("comments");
                        // let reply_collection =
                        //     state.db.database(APP_NAME).collection::<Reply>("replies");

                        // let _ = increase_likes_comment(comment_collection, target_object_id).await;

                        // let _ = increase_likes_reply(reply_collection, target_object_id).await;

                        // if let Ok(Some(comment)) = comment_collection
                        //     .find_one(doc! {"_id": target_object_id}, None)
                        //     .await
                        // {
                        //     let new_value = if comment.likes == 9_223_372_036_854_775_807 {
                        //         comment.likes
                        //     } else {
                        //         comment.likes + 1
                        //     };
                        //     let _ = comment_collection
                        //         .find_one_and_update(
                        //             doc! {"_id": comment.id},
                        //             doc! {"$set": doc! {"likes": new_value as i64}},
                        //             None,
                        //         )
                        //         .await;
                        // }

                        // if let Ok(Some(reply)) = reply_collection
                        //     .find_one(doc! {"_id": target_object_id}, None)
                        //     .await
                        // {
                        //     let new_value = if reply.likes == 9_223_372_036_854_775_807 {
                        //         reply.likes
                        //     } else {
                        //         reply.likes + 1
                        //     };
                        //     let _ = reply_collection
                        //         .find_one_and_update(
                        //             doc! {"_id": reply.id},
                        //             doc! {"$set": doc!{"likes": new_value as i64}},
                        //             None,
                        //         )
                        //         .await;
                        // }
                        return HttpResponse::Ok().finish();
                    } else {
                        return HttpResponse::BadRequest().finish();
                    }
                }
                _ => return HttpResponse::BadRequest().finish(),
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

    if let (Some(target_id), Some(target)) = (qs.get("target_id"), qs.get("target")) {
        if let Ok(target_object_id) = ObjectId::parse_str(target_id) {
            match target {
                "comment" | "reply" | "artist" | "album" => {
                    let like_collection = state.db.database(APP_NAME).collection::<Like>("likes");
                    if let Ok(Some(res)) = like_collection
                        .find_one_and_delete(
                            doc! {"user_id": user_id, "target_id": target_object_id},
                            None,
                        )
                        .await
                    {
                        match target {
                            "comment" => {
                                let comment_collection = state
                                    .db
                                    .database(APP_NAME)
                                    .collection::<Comment>("comments");
                                let _ =
                                    reduce_likes_comment(comment_collection, res.target_id).await;
                            }
                            "reply" => {
                                let reply_collection =
                                    state.db.database(APP_NAME).collection::<Reply>("replies");

                                let _ = reduce_likes_reply(reply_collection, res.target_id).await;
                            }
                            _ => {}
                        }

                        // if let Ok(Some(comment)) = comment_collection
                        //     .find_one(doc! {"_id": res.target_id}, None)
                        //     .await
                        // {
                        //     let new_value = match comment.likes.checked_sub(1) {
                        //         Some(v) => v,
                        //         None => 0,
                        //     };
                        //     let _ = comment_collection
                        //         .find_one_and_update(
                        //             doc! {"_id": comment.id},
                        //             doc! {"$set": doc! {"likes": new_value as i64}},
                        //             None,
                        //         )
                        //         .await;
                        // }

                        // if let Ok(Some(reply)) = reply_collection
                        //     .find_one(doc! {"_id": res.target_id}, None)
                        //     .await
                        // {
                        //     let new_value = match reply.likes.checked_sub(1) {
                        //         Some(v) => v,
                        //         None => 0,
                        //     };
                        //     let _ = reply_collection
                        //         .find_one_and_update(
                        //             doc! {"_id": reply.id},
                        //             doc! {"$set": doc!{"likes": new_value as i64}},
                        //             None,
                        //         )
                        //         .await;
                        // }

                        return HttpResponse::Ok().finish();
                    } else {
                        return HttpResponse::BadRequest().finish();
                    }
                }
                _ => return HttpResponse::BadRequest().finish(),
            }
        }
    }

    HttpResponse::Unauthorized().finish()
}

#[inline]
async fn increase_likes_comment(comment_collection: Collection<Comment>, target_id: ObjectId) {
    if let Ok(Some(comment)) = comment_collection
        .find_one(doc! {"_id": target_id}, None)
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
}

#[inline]
async fn increase_likes_reply(reply_collection: Collection<Reply>, target_id: ObjectId) {
    if let Ok(Some(reply)) = reply_collection
        .find_one(doc! {"_id": target_id}, None)
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
}

#[inline]
async fn reduce_likes_comment(comment_collection: Collection<Comment>, target_id: ObjectId) {
    if let Ok(Some(comment)) = comment_collection
        .find_one(doc! {"_id": target_id}, None)
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
}

#[inline]
async fn reduce_likes_reply(reply_collection: Collection<Reply>, target_id: ObjectId) {
    if let Ok(Some(reply)) = reply_collection
        .find_one(doc! {"_id": target_id}, None)
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
}
