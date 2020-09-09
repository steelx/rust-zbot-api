// mod handlers
mod user;
use actix_web::{web, web::ServiceConfig, HttpResponse};

use crate::errors::AppError;
use user::create_user;

type AppResult<T> = Result<T, AppError>;
type AppResponse = AppResult<HttpResponse>;

pub fn app_config(config: &mut ServiceConfig) {
    let ping_resource = web::resource("/")
        .route(web::get().to(ping));

    let signup = web::resource("/signup").route(web::post().to(create_user));

    config.service(ping_resource).service(signup);
}

pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}