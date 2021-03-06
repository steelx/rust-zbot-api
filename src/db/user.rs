// db user
use crate::{
    config::crypto::CryptoService,
    errors::AppError,
    models::user::UpdateProfile,
    models::user::{NewUser, User},
};
use actix_web::{web::Data, FromRequest};
use color_eyre::Result;
use futures::future::{ready, Ready};
use sqlx::{postgres::PgQueryAs, PgPool};
use std::{ops::Deref, sync::Arc};
use tracing::instrument;
use uuid::Uuid;
use crate::models::user::{UserLocation, UpdateUserLocation};

pub struct UserRepository {
    pool: Arc<PgPool>,
}

impl UserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        UserRepository { pool }
    }

    pub async fn create(&self, new_user: NewUser, crypto_service: &CryptoService) -> Result<User> {
        let password_hash = crypto_service.hash_password(new_user.password).await?;
        //insert into DB uses trait PgQueryAs
        let user = sqlx::query_as::<_, User>(
            "insert into users (username, email, password_hash) values ($1, $2, $3) returning *",
        )
        .bind(new_user.username)
        .bind(new_user.email)
        .bind(password_hash)
        .fetch_one(&*self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_profile(&self, user_id: Uuid, profile: UpdateProfile) -> Result<User> {

        if let Some(mut locations) = profile.locations {
            println!("updating location {:#?}", &locations);
            let update_location = UpdateUserLocation::create_from_vec(locations.as_mut());

            let could_have_location = self.find_location_by_user_id(user_id)
                .await?;

            match could_have_location {
                Some(location) => {
                    self.update_user_location(location.id, update_location).await?;
                },
                None => {
                    self.create_user_location(user_id, update_location).await?;
                },
            }
        }

        let user = sqlx::query_as::<_, User>(
            "update users set full_name = $2, bio = $3, image = $4 where id = $1 returning *",
        )
        .bind(user_id)
        .bind(profile.full_name)
        .bind(profile.bio)
        .bind(profile.image)
        .fetch_one(&*self.pool)
        .await?;

        Ok(user)
    }

    #[instrument(skip(self))]
    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let maybe_user = sqlx::query_as::<_, User>("select * from users where username = $1")
            .bind(username)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(maybe_user)
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let maybe_user = sqlx::query_as::<_, User>("select * from users where id = $1")
            .bind(id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(maybe_user)
    }

    pub async fn create_user_location(&self, user_id: Uuid, update_location: UpdateUserLocation) -> Result<UserLocation> {
        let user_location = sqlx::query_as::<_, UserLocation>(
            "insert into user_locations (user_id, street, city, state, country) values ($1, $2, $3, $4, $5) returning *",
        )
            .bind(user_id)
            .bind(update_location.street)
            .bind(update_location.city)
            .bind(update_location.state)
            .bind(update_location.country)
            .fetch_one(&*self.pool)
            .await?;

        Ok(user_location)
    }

    pub async fn update_user_location(&self, location_id: Uuid, location: UpdateUserLocation) -> Result<UserLocation> {
        let user_location = sqlx::query_as::<_, UserLocation>(
            "update user_locations set street = $2, city = $3, state = $4, country = $5 where id = $1 returning *",
        )
            .bind(location_id)
            .bind(location.street)
            .bind(location.city)
            .bind(location.state)
            .bind(location.country)
            .fetch_one(&*self.pool)
            .await?;

        Ok(user_location)
    }

    #[instrument(skip(self))]
    pub async fn find_location_by_user_id(&self, user_id: Uuid) -> Result<Option<UserLocation>> {
        let maybe_user = sqlx::query_as::<_, UserLocation>("select * from user_locations where user_id = $1")
            .bind(user_id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(maybe_user)
    }
}

impl FromRequest for UserRepository {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    #[instrument(skip(req, payload))]
    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let pool_result = Data::<PgPool>::from_request(req, payload).into_inner();

        match pool_result {
            Ok(pool) => ready(Ok(UserRepository::new(pool.deref().clone()))),
            _ => ready(Err(AppError::NOT_AUTHORIZED.default())),
        }
    }
}
