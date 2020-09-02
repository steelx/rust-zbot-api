//discord bot API
mod config;

//local modules
use crate::config::Config;


//external packages
use color_eyre::Result;
use actix_web::{HttpServer, App, middleware::Logger};

#[actix_rt::main]
async fn main() -> Result<()> {
    
    let config = Config::from_env().expect("Failed to load env configuration");

    println!("Server running at {}:{}", config.host, config.port);
    HttpServer::new(move || {
        App::new()
        .wrap(Logger::default())
    })
    .bind(format!("{}:{}", config.host, config.port))?
    .run()
    .await?;

    
    Ok(())
}
