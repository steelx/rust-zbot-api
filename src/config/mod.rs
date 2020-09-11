//module config
pub mod crypto;

use dotenv::dotenv;
use color_eyre::Result;
use eyre::WrapErr;
use serde::Deserialize;
use tracing::{info, instrument};//macro
use tracing_subscriber::EnvFilter;
use sqlx::postgres::PgPool;
use std::{sync::Arc, time::Duration};
use crypto::CryptoService;


#[derive(Deserialize)]
pub struct AuthConfig {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UbiConfig {
    pub appid: String,// ubi-appid
    pub authorization_prefix: String,// ubi_v1 t=ACCESS_TOKEN
    pub spaces_id_pc: String,
    pub spaces_id_xbox: String,
    pub spaces_id_ps4: String,
    pub sandbox_pc: String,
    pub sandbox_xbox: String,
    pub sandbox_ps4: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub auth: AuthConfig,
    pub ubi: UbiConfig,
    pub database_url: String,
    pub host: String,
    pub port: i32,
    pub secret_key: String,
    pub jwt_secret: String,
}

impl Config {

    #[instrument]
    pub fn from_env() -> Result<Self> {
        dotenv().ok();

        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())//env RUST_LOG
            .init();
        info!("Loading configuration..");

        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::default())?;

        //WrapErr: trait can be used on Result context
        cfg.try_into()
            .context("Loading configuration from env")
    }

    pub async fn db_pool(&self) -> Result<PgPool> {
        info!("Creating database connecting pool.");

        PgPool::builder()
            .connect_timeout(Duration::from_secs(30))
            .build(&self.database_url)
            .await
            .context("Creating database connection pool!")//context converts Result error to eyre Report
    }

    pub fn crypto_service(&self) -> CryptoService {
        CryptoService {
            key: Arc::new(self.secret_key.clone()),
            jwt_secret: Arc::new(self.jwt_secret.clone()),
        }
    }
}