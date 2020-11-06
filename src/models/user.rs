use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
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

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserWithLocation {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub locations: Vec<String>,
}

impl User {
    pub fn with_locations(&self, locations: Vec<String>) -> UserWithLocation {
        UserWithLocation {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            full_name: self.full_name.clone(),
            bio: self.bio.clone(),
            image: self.image.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            locations,
        }
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserLocation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
}

impl UserLocation {
    pub fn to_array(&self) -> Vec<String> {
        let mut locations: Vec<String> = Vec::new();

        if let Some(street) = self.street.clone() {
            locations.push(street);
        }
        if let Some(city) = self.city.clone() {
            locations.push(city);
        }
        if let Some(state) = self.state.clone() {
            locations.push(state);
        }
        if let Some(country) = self.country.clone() {
            locations.push(country);
        }

        locations
    }
}

//add it to DB
#[derive(Debug, Deserialize, Validate)]
pub struct NewUser {
    #[validate(length(min = 4))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 5))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfile {
    #[validate(length(min = 4))]
    pub full_name: Option<String>,
    pub bio: Option<String>,
    #[validate(url)]
    pub image: Option<String>,
    pub locations: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserLocation {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
}

impl UpdateUserLocation {
    pub fn create_from_vec(locations: &mut Vec<String>) -> UpdateUserLocation {
        let country: Option<String> = locations.pop();
        let state: Option<String> = locations.pop();
        let city: Option<String> = locations.pop();
        let street: Option<String> = locations.pop();

        // if locations.len() == 4 {
        //     street = Some(locations[0].to_string());
        //     city = Some(locations[1].to_string());
        //     state = Some(locations[2].to_string());
        //     country = Some(locations[3].to_string());
        // } else if locations.len() == 3 {
        //     city = Some(locations[0].to_string());
        //     state = Some(locations[1].to_string());
        //     country = Some(locations[2].to_string());
        // } else if locations.len() == 2 {
        //     state = Some(locations[0].to_string());
        //     country = Some(locations[1].to_string());
        // } else if locations.len() == 1 {
        //     country = Some(locations[0].to_string());
        // }

        UpdateUserLocation {
            street,
            city,
            state,
            country,
        }
    }
}
