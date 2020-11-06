//handlers user

use super::{auth::AuthenticatedUser, AppResponse};
use crate::{
    config::crypto::CryptoService,
    db,
    db::user::UserRepository,
    errors::AppError,
    models::user::{NewUser, UpdateProfile, User},
};

use actix_web::{
    web::{Data, Json},
    HttpResponse,
};

use color_eyre::Result;
use sqlx::{error::DatabaseError, postgres::PgError};
use tracing::{debug, instrument};
use validator::Validate;

#[instrument(skip(user, repository, crypto_service))]
pub async fn create_user(
    user: Json<NewUser>,
    repository: UserRepository,
    crypto_service: Data<CryptoService>,
) -> AppResponse {
    match user.validate() {
        Ok(_) => Ok(()),
        Err(e) => {
            let error_map = e.field_errors();

            let message = if error_map.contains_key("username") {
                format!("Invalid username. \"{}\" is too short.", user.username)
            } else if error_map.contains_key("email") {
                format!("Invalid email address \"{}\"", user.email)
            } else if error_map.contains_key("password") {
                "Invalid password. Too short".to_string()
            } else {
                "Invalid input.".to_string()
            };

            Err(AppError::INVALID_INPUT.message(message))
        }
    }?;

    let result: Result<User> = repository.create(user.0, crypto_service.as_ref()).await;

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

#[instrument[skip(user, repository)]]
pub async fn me(user: AuthenticatedUser, repository: UserRepository) -> AppResponse {
    let user = repository
        .find_by_id(user.0)
        .await?
        .ok_or(AppError::INTERNAL_ERROR)?;

    Ok(HttpResponse::Ok().json(user))
}

#[instrument[skip(user, repository)]]
pub async fn update_profile(
    user: AuthenticatedUser,
    repository: UserRepository,
    profile: Json<UpdateProfile>,
) -> AppResponse {
    // valid update_profile has all required fields
    match profile.validate() {
        Ok(_) => Ok(()),
        Err(e) => {
            let error_map = e.field_errors();

            let message = if error_map.contains_key("image") {
                format!(
                    "Invalid image. \"{}\" is not a valid url.",
                    profile.image.as_deref().unwrap()
                )
            } else {
                "Invalid input.".to_string()
            };

            Err(AppError::INVALID_INPUT.message(message))
        }
    }?;

    // find user from Auth token
    let user = repository
        .find_by_id(user.0)
        .await?
        .ok_or(AppError::INTERNAL_ERROR)?;

    //update to DB
    let updated_user = repository.update_profile(user.id, profile.0).await?;

    match repository.find_location_by_user_id(user.id).await {
        Ok(user_location) => match user_location {
            Some(location) => {
                let user_with_locations = updated_user.with_locations(location.to_array());
                Ok(HttpResponse::Ok().json(user_with_locations))
            }
            None => Ok(HttpResponse::Ok().json(user)),
        },
        Err(_) => Ok(HttpResponse::Ok().json(user)),
    }
}
