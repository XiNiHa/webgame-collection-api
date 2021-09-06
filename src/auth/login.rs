use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use webgame_collection_api_macros::Error;

use super::{password_data::PasswordData, AuthMethodType};

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

pub fn create_access_token(uuid: &Uuid, secret: &[u8]) -> Option<String> {
    let header = Header::new(Algorithm::HS512);
    let claims = Claims {
        sub: uuid.to_string(),
        exp: Utc::now()
            .checked_add_signed(Duration::hours(1))?
            .timestamp() as usize,
    };

    jsonwebtoken::encode(&header, &claims, &EncodingKey::from_secret(secret)).ok()
}
