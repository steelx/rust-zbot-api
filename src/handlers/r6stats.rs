
use super::{AppResponse, auth::AuthenticatedUser};
use crate::{
    db, 
    errors::AppError,
};

use actix_web::{
    web::{Data, Json},
    HttpResponse,
};

use color_eyre::Result;
use sqlx::{error::DatabaseError, postgres::PgError};
use tracing::{debug, instrument};
use validator::Validate;


pub async fn stats() -> AppResponse {
    
    
    Ok(HttpResponse::Ok().json("pending"))
}

