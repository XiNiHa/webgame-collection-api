use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use webgame_collection_api_macros::Error;

use crate::schema::types::user::LoginResult;

use super::{password_data::PasswordData, AuthMethodType, REF_TOKEN_REDIS_KEY};

#[derive(Error)]
pub enum LoginError {
    #[error(message = "Database error")]
    DbError(sqlx::Error),
    #[error(message = "Password not provided while required for the requested method type")]
    PasswordNotProvided,
    #[error(message = "No matching auth method found")]
    MethodNotFound,
    #[error(message = "Invalid auth method data detected")]
    InvalidMethodData,
    #[error(message = "Wrong password")]
    WrongPassword,
    #[error(message = "Token creation failed")]
    TokenCreationFailed,
}

pub async fn verify_auth_method(
    pool: &PgPool,
    auth_type: AuthMethodType,
    identifier: String,
    password: Option<String>,
) -> Result<Uuid, LoginError> {
    let method = sqlx::query!(
        r#"
        SELECT
            user_id,
            extra_info
        FROM public.user_auth_method
        WHERE type = $1 AND identifier = $2
        "#,
        &auth_type as &AuthMethodType,
        identifier,
    )
    .fetch_optional(pool)
    .await
    .map_err(LoginError::DbError)?
    .ok_or(LoginError::MethodNotFound)?;

    let password_data: Option<PasswordData> = match method.extra_info {
        Some(data) => {
            Some(serde_json::from_value(data).map_err(|_| LoginError::InvalidMethodData)?)
        }
        None => None,
    };

    if auth_type == AuthMethodType::Email
        && !password_data
            .ok_or(LoginError::InvalidMethodData)?
            .verify(&password.ok_or(LoginError::PasswordNotProvided)?)
    {
        Err(LoginError::WrongPassword)
    } else {
        Ok(method.user_id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn create_login_result(
    uuid: &Uuid,
    secret: &[u8],
    refresh_token_size: usize,
) -> Result<LoginResult, LoginError> {
    let header = Header::new(Algorithm::HS512);
    let claims = Claims {
        sub: uuid.to_string(),
        exp: Utc::now()
            .checked_add_signed(Duration::hours(1))
            .ok_or(LoginError::TokenCreationFailed)?
            .timestamp() as usize,
    };

    let access_token = jsonwebtoken::encode(&header, &claims, &EncodingKey::from_secret(secret))
        .map_err(|_| LoginError::TokenCreationFailed)?;

    let mut buf: Vec<u8> = vec![0; refresh_token_size];
    let rng = SystemRandom::new();
    rng.fill(&mut buf)
        .map_err(|_| LoginError::TokenCreationFailed)?;
    let refresh_token = base64::encode(&buf);

    Ok(LoginResult {
        access_token,
        refresh_token,
    })
}
