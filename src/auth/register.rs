use sqlx::PgPool;

use crate::{
    error::Error,
    schema::types::{
        scalars::{DateTimeScalar, UuidScalar},
        user::{User, UserRegisterInput},
    },
};

use super::password_data::PasswordData;

#[derive(Debug)]
pub enum RegistrationError {
    DbError(sqlx::Error),
    UserAlreadyExists,
    PasswordDataNotPresent,
    PasswordDataInvalid,
}

impl Error for RegistrationError {
    fn message(&self) -> String {
        match self {
            RegistrationError::DbError(_) => "Database error",
            RegistrationError::UserAlreadyExists => "User already exists",
            RegistrationError::PasswordDataNotPresent => "Password data not present",
            RegistrationError::PasswordDataInvalid => "Password data invalid",
        }
        .to_owned()
    }

    fn code(&self) -> String {
        format!("RegistrationError::{:?}", self)
    }
}

#[derive(sqlx::Type, PartialEq)]
#[sqlx(type_name = "auth_method_type", rename_all = "lowercase")]
pub enum AuthMethodType {
    Email,
    Kakao,
    Google,
    Facebook,
}

pub async fn register(
    pool: &PgPool,
    input: UserRegisterInput,
    auth_type: AuthMethodType,
    identifier: String,
    password_data: Option<PasswordData>,
) -> Result<User, RegistrationError> {
    if auth_type == AuthMethodType::Email && password_data.is_none() {
        return Err(RegistrationError::PasswordDataNotPresent);
    }

    let existing_user = sqlx::query!(
        r#"
        SELECT * FROM public.user
        WHERE nickname = $1 OR email = $2
        "#,
        input.nickname,
        input.email
    )
    .fetch_optional(pool)
    .await
    .map_err(RegistrationError::DbError)?;

    if existing_user.is_some() {
        return Err(RegistrationError::UserAlreadyExists);
    }

    let data = match password_data {
        Some(data) => match serde_json::to_value(data) {
            Ok(data) => Ok(Some(data)),
            Err(_) => Err(RegistrationError::PasswordDataInvalid),
        },
        None => Ok(None),
    }?;

    let mut tx = pool.begin().await.map_err(RegistrationError::DbError)?;

    let user = sqlx::query!(
        r#"
        INSERT INTO public.user (id, nickname, email, registered_at)
        VALUES (uuid_generate_v4(), $1, $2, CURRENT_TIMESTAMP)
        RETURNING *
        "#,
        input.nickname,
        input.email,
    )
    .fetch_one(&mut tx)
    .await
    .map_err(RegistrationError::DbError)?;

    let _auth_result = sqlx::query!(
        r#"
        INSERT INTO public.user_auth_method (user_id, type, identifier, extra_info)
        VALUES ($1, $2, $3, $4)
        "#,
        user.id,
        auth_type as AuthMethodType,
        identifier,
        data
    )
    .execute(&mut tx)
    .await
    .map_err(RegistrationError::DbError)?;

    tx.commit().await.map_err(RegistrationError::DbError)?;

    Ok(User {
        id: UuidScalar(user.id),
        nickname: user.nickname,
        email: user.email,
        registered_at: DateTimeScalar(user.registered_at),
        deleted_at: user.deleted_at.map(DateTimeScalar),
    })
}
