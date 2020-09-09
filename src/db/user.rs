// db user
use actix_web::{web::Data, FromRequest};
use futures::future::{Ready, ready};
use color_eyre::Result;
use sqlx::{PgPool, postgres::PgQueryAs};
use std::{sync::Arc, ops::Deref};
use crate::{config::crypto::CryptoService, models::user::{User, NewUser}, errors::AppError};
use tracing::instrument;

pub struct UserRepository {
    pool: Arc<PgPool>,
}

impl UserRepository {
    
    pub fn new(pool: Arc<PgPool>) -> Self {
        UserRepository {
            pool,
        }
    }

    pub async fn create(&self, new_user: NewUser, crypto_service: &CryptoService) -> Result<User> {
        let password_hash = crypto_service.hash_password(new_user.password).await?;
        
        //insert into DB uses trait PgQueryAs
        let user = sqlx::query_as::<_, User>(
            "insert into users (username, email, password_hash) values ($1, $2, $3) returning *"
        )
        .bind(new_user.username)
        .bind(new_user.email)
        .bind(password_hash)
        .fetch_one(&*self.pool)
        .await?;

        Ok(user)
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