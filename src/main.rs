
mod apis;
mod collections;
mod error;
use actix_cors::Cors;
use actix_web::web::service;
use actix_web::{get, http, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

use apis::reply::{get_replies, create_reply, delete_reply, update_reply};
use apis::user::{get_user, login, register, search_user};
use apis::comment::{create_comment, get_comments, delete_comment, update_comment, find_comment_by_id};
use apis::like::{create_like, is_like, delete_like};
use chrono::Utc;
use collections::{Session, User, Like};
use dotenv::dotenv;
use mongodb::{bson::doc, options::ClientOptions, options::IndexOptions, Client, IndexModel};
use std::error::Error;
use std::time::Duration;

const APP_NAME: &str = "musicView";
const SESSION_LIFE: i64 = 1800;
const SESSION_LIFE_GUEST: i64 = 86400;
const SESSION_CLEANING_FREQUENCY: u64 = 1800;
const SERVER_PORT: u16 = 4000;

struct AppState {
    db: Client,
}

async fn user_collection_init(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(options.clone())
        .build();

    let _collection = client
        .database(APP_NAME)
        .collection::<User>("users")
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");

    let model = IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(options)
        .build();

    let _collection = client
        .database(APP_NAME)
        .collection::<User>("users")
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}

async fn like_collection_init(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "target_id": 1, "user_id": 1 })
        .options(options)
        .build();

    let _collection = client
        .database(APP_NAME)
        .collection::<Like>("likes")
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .map_err(|e| format!("Error: {e}"))?;
    let session_collection = client.database(APP_NAME).collection::<Session>("sessions");

    tokio::spawn(async move {
        loop {
            let cur_time = Utc::now().timestamp();
            match session_collection
                .delete_many(doc! { "expiration_date": { "$lt": cur_time } }, None)
                .await
            {
                Ok(res) => {
                    dbg!(res);
                }
                Err(e) => {
                    dbg!(e);
                }
            }

            tokio::time::sleep(Duration::from_secs(SESSION_CLEANING_FREQUENCY)).await;
        }
    });

    user_collection_init(&client).await;
    like_collection_init(&client).await;

    let app_state = web::Data::new(AppState { db: client });

    HttpServer::new(move || {
        let origin = std::env::var("CLIENT_ORIGIN").expect("Client origin should be set");
        let cors = Cors::default()
            .allowed_origin(&origin)
            // .allow_any_origin()
            //   .allowed_origin_fn(|origin, _req_head| {
            //       origin.as_bytes().ends_with(b".rust-lang.org")
            //   })
            .allowed_methods(vec!["GET", "POST", "DELETE", "PUT"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                // http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                // http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            ])
            .allowed_header(http::header::CONTENT_TYPE);
        // .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(login)
            .service(register)
            .service(get_user)
            .service(search_user)
            .service(create_comment)
            .service(get_comments)
            .service(delete_comment)
            .service(update_comment)
            .service(get_replies)
            .service(create_reply)
            .service(delete_reply)
            .service(update_reply)
            .service(is_like)
            .service(create_like)
            .service(delete_like)
            .service(find_comment_by_id)
        // .service(email_exists)
        // .service(index)
        // .service(create_user)
        // .service(username_exists)
        // .service(login_with_password)
        // .service(login_with_session)
        // .service(delete_user)
    })
    .keep_alive(Duration::from_secs(25))
    .bind(("0.0.0.0", SERVER_PORT))?
    .run()
    .await?;

    Ok(())
}
