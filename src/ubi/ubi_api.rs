use crate::{config::UbiConfig, errors::AppError, handlers::AppResponse};
use crate::models::ubi_user::{NewUbiUser};
use crate::UbiUserRepository;
use crate::db;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
//use serde_json::{json};
use std::collections::HashMap;
use color_eyre::Result;
use sqlx::{error::DatabaseError, postgres::PgError};
use tracing::{debug, instrument};
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONNECTION, CONTENT_LENGTH, CONTENT_TYPE,
    USER_AGENT,
};
//use tracing::{debug, instrument};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub platform_type: String,
    pub ticket: String,
    pub profile_id: String,
    pub user_id: String,
    pub name_on_platform: String,
    pub expiration: String, //2020-08-26T16:46:59.4772040Z
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub profile_id: String,
    pub is_friend: bool,
    pub avatar: String,
    pub member_since: String, //"2017-09-17T03:01:35.86Z"
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RankStats {
    pub players: HashMap<String, PlayerStats>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlayerStats {
    pub max_mmr: f32,
    pub skill_mean: f32,
    pub deaths: i32,
    pub profile_id: String,
    pub next_rank_mmr: f32,
    pub rank: i32,
    pub max_rank: i32,
    pub board_id: String,
    pub skill_stdev: f32,
    pub kills: i32,
    pub last_match_skill_stdev_change: f32,
    pub update_time: String, //"2020-08-23T12:05:48.558000+00:00"
    pub last_match_mmr_change: f32,
    pub abandons: i32,
    pub season: i32,
    pub top_rank_position: i32,
    pub last_match_skill_mean_change: f32,
    pub mmr: f32,
    pub previous_rank_mmr: f32,
    pub last_match_result: i32,
    pub wins: i32,
    pub region: String, //"apac"
    pub losses: i32,
}

#[derive(Clone)]
pub struct UbiApi {
    email: String,
    password: String,
    pub ubi_config: UbiConfig,
    pub expiration: String,
    pub authorization: String,
    client: reqwest::Client,
}

impl UbiApi {
    pub fn new(
        email: &str,
        password: &str,
        reqwest_client: reqwest::Client,
        ubi_config: UbiConfig,
    ) -> Self {
        UbiApi {
            email: email.to_string(),
            password: password.to_string(),
            ubi_config,
            client: reqwest_client,
            authorization: String::from(""),
            expiration: String::from(""),
        }
    }

    pub async fn login(&mut self, repository: UbiUserRepository) -> AppResponse {
        let mut post_payload = HashMap::new();
        post_payload.insert("rememberMe".to_string(), true);

        let request_url = "https://public-ubiservices.ubi.com/v3/profiles/sessions";
        let response = self
            .client
            .post(request_url)
            .json(&post_payload)
            .headers(self.construct_headers(true))
            .basic_auth(self.email.clone(), Some(self.password.clone()))
            .send()
            .await
            .map_err(|op| {
                debug!("Error login in to UBI session. {:?}", op);
                AppError::INTERNAL_ERROR.default()
            })?;

        //let status_code = response.status().clone();
        //println!("login status: {}", status_code);
        // println!("BODY: {:#?}", response.text().await?);
        let body = response.json::<Session>().await.map_err(|op| {
            debug!("Error reading UBI session response. {:?}", op);
            AppError::INTERNAL_ERROR.default()
        })?;

        //println!("{:#?}", body.clone());

        self.authorization = format!(
            "{}{}",
            self.ubi_config.authorization_prefix.clone(),
            body.ticket
        );
        self.expiration = body.expiration;

        let ubi_user = NewUbiUser {
            email: self.email.clone(),
            password: self.password.clone(),
            token: self.authorization.clone(),
            expiration: self.expiration.clone(),
        };
        let result = repository.create(ubi_user).await;

        match result {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
            Err(e) => {
                let pg_error: &PgError = e.root_cause().downcast_ref::<PgError>().ok_or_else(|| {
                    debug!("Error creating user. {:?}", e);
                    AppError::INTERNAL_ERROR
                })?;
    
                let error = match (pg_error.code(), pg_error.column_name()) {
                    (Some(db::UNIQUE_VIOLATION_CODE), Some("email")) => {
                        AppError::INVALID_INPUT.message("Email address already exists.".to_string())
                    }
                    (Some(db::UNIQUE_VIOLATION_CODE), Some("username")) => {
                        AppError::INVALID_INPUT.message("Username already exists.".to_string())
                    }
                    (Some(db::UNIQUE_VIOLATION_CODE), None) => {
                        AppError::INVALID_INPUT.message("Username or email already exists.".to_string())
                    }
                    _ => {
                        debug!("Error creating user. {:?}", pg_error);
                        AppError::INTERNAL_ERROR.into()
                    }
                };
                Err(error)
            }
        }
    }

    fn construct_headers(&self, login: bool) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "ubi-appid",
            HeaderValue::from_str(self.ubi_config.appid.as_str()).unwrap(),
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:80.0) Gecko/20100101 Firefox/80.0",
            ),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(CONTENT_LENGTH, HeaderValue::from_static("0"));

        if !login {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(self.authorization.as_str()).unwrap(),
            );
        }

        headers
    }

    pub async fn find_profile(
        &mut self,
        username: String,
        platform_type: String,
    ) -> Result<Profile, Box<dyn std::error::Error>> {
        // let url = reqwest::Url::parse_with_params("https://public-ubiservices.ubi.com/v2/profiles",
        //     &[("platformType", platform_type), ("nameOnPlatform", username)])?;
        println!("platform_type: {}", platform_type);

        let url_str = format!("https://public-ubiservices.ubi.com/v1/profiles/me/club/aggregation/website/otherProfile/{}", username);
        let url = reqwest::Url::parse(&url_str)?;

        let response = self
            .client
            .get(url)
            .headers(self.construct_headers(false))
            .send()
            .await?;
        //
        println!("findProfile: {}", response.status());
        if response.status() == 404 {
            return Ok(Profile {
                profile_id: String::from(""),
                is_friend: false,
                avatar: String::from(""),
                member_since: String::from(""),
            });
        }

        let body = response.json::<Profile>().await?;
        Ok(body)
    }

    pub async fn find_rank_stats(
        &self,
        profile_id: String,
    ) -> Result<PlayerStats, Box<dyn std::error::Error>> {
        let url = reqwest::Url::parse_with_params("https://public-ubiservices.ubi.com/v1/spaces/5172a557-50b5-4665-b7db-e3f2e8c5041d/sandboxes/OSBOR_PC_LNCH_A/r6karma/players", 
            &[("board_id", "pvp_ranked"), ("profile_ids", &profile_id), ("region_id", "apac"), ("season_id", "-1")])?;

        let response = self
            .client
            .get(url)
            .headers(self.construct_headers(false))
            .send()
            .await?;

        let body = response.json::<RankStats>().await?;
        let stats = body.players.get(&profile_id).expect("Profile id not found");
        Ok(stats.to_owned())
    }

    //find_level
    //https://public-ubiservices.ubi.com/v1/spaces/5172a557-50b5-4665-b7db-e3f2e8c5041d/sandboxes/OSBOR_PC_LNCH_A/r6playerprofile/playerprofile/progressions?profile_ids=b5072e90-ad85-4bd8-9d18-e0bfe5f2aba5
}
