use sqlx::PgPool;

use crate::error::Error;

use super::{password_data::PasswordData, AuthMethodType};

#[derive(Debug)]
pub enum LoginError {
    DbError(sqlx::Error),
    PasswordNotProvided,
    MethodNotFound,
    InvalidMethodData,
    WrongPassword,
}

impl Error for LoginError {
    fn message(&self) -> String {
        match self {
            LoginError::DbError(_) => "Database error",
            LoginError::PasswordNotProvided => {
                "Password not provided while required for the requested method type"
            }
            LoginError::MethodNotFound => "No matching auth method found",
            LoginError::InvalidMethodData => "Invalid auth method data detected",
            LoginError::WrongPassword => "Wrong password",
        }
        .to_owned()
    }

    fn code(&self) -> String {
        format!("LoginError::{:?}", self)
    }
}

pub async fn verify_auth_method(
    pool: &PgPool,
    auth_type: AuthMethodType,
    identifier: String,
    password: Option<String>,
) -> Result<bool, LoginError> {
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
        Some(data) => Some(serde_json::from_value(data).map_err(|_| LoginError::InvalidMethodData)?),
        None => None
    };

    if auth_type == AuthMethodType::Email
        && !password_data
            .ok_or(LoginError::InvalidMethodData)?
            .verify(&password.ok_or(LoginError::PasswordNotProvided)?)
    {
        Err(LoginError::WrongPassword)
    } else {
        Ok(true)
    }
}
