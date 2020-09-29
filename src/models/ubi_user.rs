use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

//retrive from DB
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UbiUser {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub token: String,
    pub expiration: String,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

//add it to DB
#[derive(Debug, Deserialize, Validate)]
pub struct NewUbiUser {
    #[validate(email)]
    pub email: String,
    pub password: String,
    pub token: String,
    pub expiration: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUbiUser {
    pub token: String,
    pub expiration: String,
}
