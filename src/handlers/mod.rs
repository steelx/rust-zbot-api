// mod handlers
use actix_web::{web, web::ServiceConfig, HttpResponse};


pub fn app_config(config: &mut ServiceConfig) {
    let ping_resource = web::resource("/")
        .route(web::get().to(ping));

    config.service(ping_resource);
}

pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().finish()
}