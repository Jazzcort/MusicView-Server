use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<ObjectId>,
    pub(crate) email: String,
    pub(crate) hash: String,
    pub(crate) salt: String,
    pub(crate) username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) role: Option<String>,
    pub(crate) artist_id: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UserUpdateForm {
    pub(crate) email: Option<String>,
    pub(crate) username: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    // session_id: String,
    pub user_id: ObjectId,
    pub expiration_date: i64,
}

#[derive(Serialize, Debug, Deserialize)]
pub(crate) struct Comment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<ObjectId>,
    pub(crate) content: String,
    pub(crate) author: ObjectId,
    pub(crate) target_id: String,
    pub(crate) likes: u64,
}

#[derive(Serialize, Debug, Deserialize)]
pub(crate) struct Reply {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<ObjectId>,
    pub(crate) content: String,
    pub(crate) author: ObjectId,
    pub(crate) likes: u64,
    pub(crate) comment_id: ObjectId,
}

#[derive(Serialize, Debug, Deserialize)]
pub(crate) struct Like {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<ObjectId>,
    pub(crate) user_id: ObjectId,
    pub(crate) target_id: ObjectId,
    pub(crate) target: String
}

#[derive(Serialize, Debug, Deserialize)]
pub(crate) struct LikeArtist {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<ObjectId>,
    pub(crate) user_id: ObjectId,
    pub(crate) artist_id: String,
}
