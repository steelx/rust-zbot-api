
use crate::ubi;
use crate::errors::AppError;

use super::{AppResponse};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use actix_web::{
    web::{Data, Json, Query},
    HttpResponse,
};


#[derive(Deserialize)]
pub struct StatsQuery {
    name_on_platform: String,
    platform_type: String,//uplay | psn | xbl
}

pub async fn stats(Query(req): Query<StatsQuery>, ubi_api: Data<ubi::ubi_api::UbiApi>) -> AppResponse {

    let ubi_profile = ubi_api.find_profile(req.name_on_platform.clone(), req.platform_type.clone()).await?;
    
    if ubi_profile.profile_id == "" {
        return Err(AppError::NOT_FOUND.message(format!("User {:?} not found on platform {:?}", req.name_on_platform, req.platform_type)));
    }
    
    let player_stats = ubi_api.find_rank_stats(ubi_profile.profile_id).await?;
    
    Ok(HttpResponse::Ok().json(player_stats))
}

