// mod handlers
mod user;
mod auth;

use actix_web::{web, web::ServiceConfig, HttpResponse};

use crate::errors::AppError;
use user::{create_user, me, update_profile};

type AppResult<T> = Result<T, AppError>;
type AppResponse = AppResult<HttpResponse>;

pub fn app_config(config: &mut ServiceConfig) {
    let ping_resource = web::resource("/")
        .route(web::get().to(ping));

    let auth = web::resource("/auth").route(web::post().to(auth::auth));
    let me = web::resource("/me")
        .route(web::get().to(me))
        .route(web::post().to(update_profile));

    let signup = web::resource("/signup").route(web::post().to(create_user));

    config.service(ping_resource).service(signup).service(auth).service(me);
}

pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().json("ping")
}