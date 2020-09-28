// db ubi_user.rs
use crate::errors::AppError;
use crate::models::ubi_user::{NewUbiUser, UbiUser, UpdateUbiUser};
use actix_web::{web::Data, FromRequest};
use color_eyre::Result;
use futures::future::{ready, Ready};
use sqlx::{postgres::PgQueryAs, PgPool};
use uuid::Uuid;
use std::{ops::Deref, sync::Arc};
use tracing::instrument;

/// UbiUserRepository
pub struct UbiUserRepository {
    pool: Arc<PgPool>,
}

impl UbiUserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        UbiUserRepository { pool }
    }

    pub async fn create(&self, new_user: NewUbiUser) -> Result<UbiUser> {
        //insert into DB uses trait PgQueryAs
        let user = sqlx::query_as::<_, UbiUser>(
            "insert into ubi_users (email, password, token, expiration) values ($1, $2, $3, $4) returning *",
        )
        .bind(new_user.email)
        .bind(new_user.password)
        .bind(new_user.token)
        .bind(new_user.expiration)
        .fetch_one(&*self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_ubi_user(&self, user_id: Uuid, profile: UpdateUbiUser) -> Result<UbiUser> {
        let user = sqlx::query_as::<_, UbiUser>(
            "update users set token = $2, expiration = $3 where id = $1 returning *",
        )
        .bind(user_id)
        .bind(profile.token)
        .bind(profile.expiration)
        .fetch_one(&*self.pool)
        .await?;

        Ok(user)
    }

    #[instrument(skip(self))]
    pub async fn find_by_email(&self, email: &str) -> Result<Option<UbiUser>> {
        let maybe_user = sqlx::query_as::<_, UbiUser>("select * from users where email = $1")
            .bind(email)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(maybe_user)
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<UbiUser>> {
        let maybe_user = sqlx::query_as::<_, UbiUser>("select * from users where id = $1")
            .bind(id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(maybe_user)
    }
}

impl FromRequest for UbiUserRepository {
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
            Ok(pool) => ready(Ok(UbiUserRepository::new(pool.deref().clone()))),
            _ => ready(Err(AppError::NOT_AUTHORIZED.default())),
        }
    }
}
