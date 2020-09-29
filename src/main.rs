#[macro_use]
extern crate validator_derive;

mod config;
mod db;
mod errors;
mod handlers;
mod models;
mod ubi;

//local modules
use crate::config::Config;
use crate::db::ubi_user::UbiUserRepository;
use crate::handlers::app_config;

use std::sync::Arc;

//external packages
use actix_web::{middleware::Logger, App, HttpServer};
use color_eyre::Result;
use tracing::info;

#[actix_rt::main]
async fn main() -> Result<()> {
    let config = Config::from_env().expect("Failed to load env configuration");
    let db_pool = config.db_pool().await.expect("Database connection failed!");
    let crypto_service = config.crypto_service().clone();

    info!("STARTING at http://{}:{}", config.host, config.port);

    let req_client = reqwest::Client::new();
    let ubi_user_db = UbiUserRepository::new(Arc::new(db_pool.clone()));
    let mut ubi_api = ubi::ubi_api::UbiApi::new(
        config.auth.email.as_str(),
        config.auth.password.as_str(),
        req_client,
        config.ubi,
    );


    ubi_api
        .login(ubi_user_db)
        .await
        .expect("UBI authentication failed!");

    // TODO: (next time)
    // now that we are logged int, ubi_users table in DB has our ubi user token
    // use that, dont login again

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(db_pool.clone())
            .data(crypto_service.clone())
            .data(ubi_api.clone())
            .configure(app_config)
    })
    .bind(format!("{}:{}", config.host, config.port))?
    .run()
    .await?;

    Ok(())
}
