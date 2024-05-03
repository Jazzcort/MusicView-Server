mod api;
mod collections;
mod error;
use actix_web::web::service;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use api::{
    create_user, delete_user, email_exists, index, login_with_password, login_with_session, username_exists
};
use chrono::Utc;
use collections::{Session, User};
use mongodb::{bson::doc, options::ClientOptions, options::IndexOptions, Client, IndexModel};
use std::error::Error;
use std::option;
use std::time::Duration;

const APP_NAME: &str = "tauriApp";
const SESSION_LIFE: i64 = 604800;
const SESSION_CLEANING_FREQUENCY: u64 = 43200;
const SERVER_PORT: u16 = 20130;

struct AppState {
    db: Client,
}

async fn user_collection_init(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "username": 1, "email": 1 })
        .options(options)
        .build();

    let _collection = client
        .database(APP_NAME)
        .collection::<User>("users")
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    let app_state = web::Data::new(AppState { db: client });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(email_exists)
            .service(index)
            .service(create_user)
            .service(username_exists)
            .service(login_with_password)
            .service(login_with_session)
            .service(delete_user)
    })
    .keep_alive(Duration::from_secs(25))
    .bind(("0.0.0.0", SERVER_PORT))?
    .run()
    .await?;

    Ok(())
}
