use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(skip_serializing)]
    pub email: String,
    #[serde(skip_serializing)]
    pub hash: String,
    #[serde(skip_serializing)]
    pub salt: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    // session_id: String,
    pub user_id: ObjectId,
    pub expiration_date: i64
}