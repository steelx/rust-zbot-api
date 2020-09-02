//module config
use dotenv::dotenv;
use color_eyre::Result;
use eyre::WrapErr;
use serde::Deserialize;


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
    pub sandbox_pc: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub auth: AuthConfig,
    pub ubi: UbiConfig,
    pub database_url: String,
    pub host: String,
    pub port: i32,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv().ok();
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::default())?;

        //WrapErr: trait can be used on Result context
        cfg.try_into()
            .context("Loading configuration from env")
    }
}