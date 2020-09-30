// mod handlers
mod auth;
mod user;
mod r6stats;

use actix_web::{web, web::ServiceConfig, HttpResponse};

use crate::errors::AppError;
use user::{create_user, me, update_profile};

pub type AppResult<T> = Result<T, AppError>;
pub type AppResponse = AppResult<HttpResponse>;

pub fn app_config(config: &mut ServiceConfig) {
    let ping_resource = web::resource("/").route(web::get().to(ping));

    let auth = web::resource("/auth").route(web::post().to(auth::auth));
    let me = web::resource("/me")
        .route(web::get().to(me))
        .route(web::post().to(update_profile));

    let signup = web::resource("/signup").route(web::post().to(create_user));

    //ubi
    let ubi_stats = web::resource("/ubi/stats").route(web::get().to(r6stats::stats));

    config
        .service(ping_resource)
        .service(signup)
        .service(auth)
        .service(me)
        .service(ubi_stats);
}

pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().json("ping")
}
