
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
    region_id: String,// apac | emea | ncsa
    platform_type: String,// uplay | psn | xbl
}

#[derive(Deserialize)]
pub struct FindXpProfiles {
    profile_id: String,
    platform_type: String,// uplay | psn | xbl
}
#[derive(Deserialize)]
pub struct FindPopulationsStatistics {
    profile_id: String,
    platform_type: String,// uplay | psn | xbl
    statistics: String,
}

pub async fn find_stats(Query(req): Query<FindStats>, ubi_api: Data<ubi::ubi_api::UbiApi>) -> AppResponse {
    
    let player_stats = ubi_api.find_rank_stats(req.profile_id, req.region_id, req.platform_type.as_str()).await?;
    
    Ok(HttpResponse::Ok().json(player_stats))
}

/// statistics <comma seperated strings>
//	casualpvp_timeplayed,casualpvp_matchwon,casualpvp_matchlost,casualpvp_matchplayed,casualpvp_kills,casualpvp_death,rankedpvp_matchwon,rankedpvp_matchlost,rankedpvp_timeplayed,rankedpvp_matchplayed,rankedpvp_kills,rankedpvp_death
pub async fn find_populations_statistics(Query(req): Query<FindPopulationsStatistics>, ubi_api: Data<ubi::ubi_api::UbiApi>) -> AppResponse {
    
    let statistics = ubi_api.find_populations_statistics(req.profile_id.as_str(), req.platform_type.as_str(), req.statistics.as_str()).await?;
    
    Ok(HttpResponse::Ok().json(statistics))
    //return response
    //{"results": {"80189261-91c0-4bf1-a5ad-81df3e64423e": 
    //{"casualpvp_matchwon:infinite": 54, 
    //"rankedpvp_timeplayed:infinite": 1358423, "rankedpvp_matchlost:infinite": 522, "casualpvp_matchlost:infinite": 39, 
    //"rankedpvp_death:infinite": 4701, "casualpvp_kills:infinite": 392, "rankedpvp_matchwon:infinite": 655, "rankedpvp_kills:infinite": 6069, "casualpvp_matchplayed:infinite": 93, "casualpvp_death:infinite": 279, "rankedpvp_matchplayed:infinite": 1194, "casualpvp_timeplayed:infinite": 69305}}}
}

/// find_player_xp_profiles
// {
//   "player_profiles": [
//     {
//       "xp": 9931,
//       "profile_id": "b5072e90-ad85-4bd8-9d18-e0bfe5f2aba5",
//       "lootbox_probability": 1250,
//       "level": 78
//     }
//   ]
// }
pub async fn find_player_xp_profiles(Query(req): Query<FindXpProfiles>, ubi_api: Data<ubi::ubi_api::UbiApi>) -> AppResponse {
    
    let player_xp_profiles = ubi_api.find_player_xp_profiles(req.profile_id, req.platform_type.as_str()).await?;
    
    Ok(HttpResponse::Ok().json(player_xp_profiles))
}

pub async fn find_profile(Query(req): Query<FindProfile>, ubi_api: Data<ubi::ubi_api::UbiApi>) -> AppResponse {

    let profiles = ubi_api.find_profile(req.name_on_platform.clone(), req.platform_type.clone()).await?;
    
    if profiles.profiles.len() == 0 {
        return Err(AppError::NOT_FOUND.message(format!("Profile {:?} not found on platform {:?}", req.name_on_platform, req.platform_type)));
    }
    
    Ok(HttpResponse::Ok().json(profiles))
}

