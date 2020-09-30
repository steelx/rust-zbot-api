use crate::{config::UbiConfig, errors::AppError, handlers::AppResponse, handlers::AppResult};
use crate::models::ubi_user::{NewUbiUser, UbiUser, UpdateUbiUser};
use crate::UbiUserRepository;
use crate::db;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
//use serde_json::{json};
use std::collections::HashMap;
use sqlx::{error::DatabaseError, postgres::PgError};
use tracing::{debug, info};
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONNECTION, CONTENT_LENGTH, CONTENT_TYPE,
    USER_AGENT, REFERER,
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

    pub fn prefix_authorization(&mut self, token: String, expiration: String) {
        self.authorization = format!(
            "{}{}",
            self.ubi_config.authorization_prefix.clone(),
            token
        );
        self.expiration = expiration;
    }

    pub async fn login(&mut self, repository: UbiUserRepository) -> AppResponse {

        let user_result = repository
            .find_by_email(self.email.as_str())
            .await?
            .ok_or_else(|| {
                debug!("User doesn't exist.");
                AppError::INVALID_CREDENTIALS
            });

        if let Ok(user) = user_result {
            info!("Found existing UBI login");

            //check expiry
            let expiry = chrono::DateTime::parse_from_rfc3339(user.expiration.as_str()).unwrap();
            if expiry > chrono::Utc::now() {
                //update login if not expired
                info!("Token not expired, updating login :)");
                self.update_login(user.clone(), repository).await?;
                return Ok(HttpResponse::Ok().json(user));
            }
        
            info!("Token expired, lets login again!");
            //you reached here, means token expired
            //let's login now below..
            //but before that lets delete DB entry for existing user
            repository.delete_by_id(user.id).await?;
        }
        
        // return Ok(HttpResponse::Ok().json(UbiUser {
        //     id: uuid::Uuid::nil(),
        //     email: "ajinkya@test.com".to_string(),
        //     password: "12345".to_string(),
        //     token: "sadad".to_string(),
        //     expiration: "soon".to_string(),
        //     created_at: chrono::NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        //     updated_at: chrono::NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        // }));

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

        self.prefix_authorization(body.ticket, body.expiration);

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
        headers.insert(REFERER, HeaderValue::from_static("https://connect.ubisoft.com"));

        if !login {
            info!("HeaderValue token: {:?}", self.authorization.clone());
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(self.authorization.as_str()).unwrap(),
            );
        }

        headers
    }

    pub async fn update_login(&mut self, ubi_user: UbiUser, repository: UbiUserRepository) -> AppResponse {
        self.authorization = ubi_user.token;
        self.expiration = ubi_user.expiration;

        self.ping_me(ubi_user.id, repository).await
    }

    pub async fn ping_me(&mut self, user_id: uuid::Uuid, repository: UbiUserRepository) -> AppResponse {

        let url = "https://public-ubiservices.ubi.com/v3/profiles/sessions";
        let response = self
            .client
            .post(url)
            .headers(self.construct_headers(false))
            .send()
            .await
            .map_err(|op| {
                debug!("Error login in to UBI session. {:?}", op);
                AppError::INTERNAL_ERROR.default()
            })?;

        // println!("BODY: {:#?}", response.text().await);

        let session = response.json::<Session>()
            .await
            .map_err(|op| {
                debug!("Error pinging me. {:?}", op);
                AppError::INTERNAL_ERROR.default()
            })?;
        
        //Ok(HttpResponse::Ok().json(body))

        //add ticket authorization_prefix
        self.prefix_authorization(session.ticket, session.expiration);

        // push prefixed token to database
        let update_user = UpdateUbiUser {
            token: self.authorization.clone(),
            expiration: self.expiration.clone(),
        };

        self.update_token_to_db(user_id, update_user, repository).await
    }

    pub async fn update_token_to_db(&mut self, user_id: uuid::Uuid, update_user: UpdateUbiUser, repository: UbiUserRepository) -> AppResponse {
        let result = repository.update_ubi_user(user_id, update_user).await;

        match result {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
            Err(e) => {
                let pg_error: &PgError = e.root_cause().downcast_ref::<PgError>().ok_or_else(|| {
                    debug!("Error updating ubi user. {:?}", e);
                    AppError::INTERNAL_ERROR
                })?;
    
                let error = match (pg_error.code(), pg_error.column_name()) {
                    _ => {
                        debug!("Error updating ubi user. {:?}", pg_error);
                        AppError::INTERNAL_ERROR.into()
                    }
                };
                Err(error)
            }
        }
    }

    pub async fn find_profile(&self, username: String, platform_type: String) -> AppResult<Profile> {
        // let url = reqwest::Url::parse_with_params("https://public-ubiservices.ubi.com/v2/profiles",
        //     &[("platformType", platform_type), ("nameOnPlatform", username)])?;
        println!("platform_type: {}", platform_type);

        let url_str = format!("https://public-ubiservices.ubi.com/v1/profiles/me/club/aggregation/website/otherProfile/{}", username);
        let url = reqwest::Url::parse(&url_str).map_err(|op| {
            debug!("Error parsing URL {:?}", op);
            AppError::INTERNAL_ERROR.default()
        })?;

        let response = self
            .client
            .get(url)
            .headers(self.construct_headers(false))
            .send()
            .await
            .map_err(|op| {
                debug!("Error ubi_api could not find username. {:?}", op);
                AppError::INTERNAL_ERROR.default()
            })?;

        if response.status() == 404 {
            return Ok(Profile {
                profile_id: String::from(""),
                is_friend: false,
                avatar: String::from(""),
                member_since: String::from(""),
            });
        }

        let body = response.json::<Profile>().await.map_err(|op| {
            debug!("Error parsing Profile {:?}", op);
            AppError::INTERNAL_ERROR.default()
        })?;

        Ok(body)
    }

    pub async fn find_rank_stats(&self, profile_id: String) -> AppResult<PlayerStats> {
        let url = reqwest::Url::parse_with_params("https://public-ubiservices.ubi.com/v1/spaces/5172a557-50b5-4665-b7db-e3f2e8c5041d/sandboxes/OSBOR_PC_LNCH_A/r6karma/players", 
            &[("board_id", "pvp_ranked"), ("profile_ids", &profile_id), ("region_id", "apac"), ("season_id", "-1")])
            .map_err(|op| {
                debug!("Error parsing URL {:?}", op);
                AppError::INTERNAL_ERROR.default()
            })?;

        let response = self
            .client
            .get(url)
            .headers(self.construct_headers(false))
            .send()
            .await
            .map_err(|op| {
                debug!("Error ubi_api could not find stats. {:?}", op);
                AppError::INTERNAL_ERROR.default()
            })?;

        let body = response.json::<RankStats>().await
            .map_err(|op| {
                debug!("Error parsing RankStats {:?}", op);
                AppError::INTERNAL_ERROR.default()
            })?;
        
        let stats = body.players.get(&profile_id).expect("Profile id not found");
        Ok(stats.to_owned())
    }

    //find_level
    //https://public-ubiservices.ubi.com/v1/spaces/5172a557-50b5-4665-b7db-e3f2e8c5041d/sandboxes/OSBOR_PC_LNCH_A/r6playerprofile/playerprofile/progressions?profile_ids=b5072e90-ad85-4bd8-9d18-e0bfe5f2aba5
}
