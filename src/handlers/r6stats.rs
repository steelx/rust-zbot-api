
use crate::ubi;
use crate::errors::AppError;

use super::{AppResponse};
use serde::{Deserialize};
use actix_web::{
    web::{Data, Query},
    HttpResponse,
};


#[derive(Deserialize)]
pub struct FindProfile {
    name_on_platform: String,
    platform_type: String,//uplay | psn | xbl
}

#[derive(Deserialize)]
pub struct FindStats {
    profile_id: String,
    region_id: String,//"apac" | emea'
    platform_type: String,//uplay | psn | xbl
}

pub async fn find_stats(Query(req): Query<FindStats>, ubi_api: Data<ubi::ubi_api::UbiApi>) -> AppResponse {
    
    let player_stats = ubi_api.find_rank_stats(req.profile_id, req.region_id, req.platform_type.as_str()).await?;
    
    Ok(HttpResponse::Ok().json(player_stats))
}

pub async fn find_profile(Query(req): Query<FindProfile>, ubi_api: Data<ubi::ubi_api::UbiApi>) -> AppResponse {

    let profiles = ubi_api.find_profile(req.name_on_platform.clone(), req.platform_type.clone()).await?;
    
    if profiles.profiles.len() == 0 {
        return Err(AppError::NOT_FOUND.message(format!("Profile {:?} not found on platform {:?}", req.name_on_platform, req.platform_type)));
    }
    
    Ok(HttpResponse::Ok().json(profiles))
}

