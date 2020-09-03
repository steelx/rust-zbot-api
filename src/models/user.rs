use uuid::Uuid;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use validator::Validate;

//retrive from DB
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,

    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}


//add it to DB
#[derive(Debug, Deserialize, Validate)]
pub struct NewUser {
    #[validate(length(min = 4))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 5))]
    pub password_hash: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfile {
    #[validate(length(min = 5))]
    pub full_name: Option<String>,
    #[validate(length(min = 5))]
    pub bio: Option<String>,
    #[validate(url)]
    pub image: Option<String>,
}