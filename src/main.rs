//discord bot API
#[macro_use]
extern crate validator_derive;


mod config;
mod handlers;
mod models;

//local modules
use crate::config::Config;
use crate::handlers::app_config;


//external packages
use tracing::info;
use color_eyre::Result;
use actix_web::{HttpServer, App, middleware::Logger};

#[actix_rt::main]
async fn main() -> Result<()> {
    
    let config = Config::from_env().expect("Failed to load env configuration");

    info!("STARTING at http://{}:{}", config.host, config.port);

    HttpServer::new(move || {
        App::new()
        .wrap(Logger::default())
        .configure(app_config)
    })
    .bind(format!("{}:{}", config.host, config.port))?
    .run()
    .await?;

    
    Ok(())
}
