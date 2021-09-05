use sqlx::PgPool;
use webgame_collection_api_macros::Error;

use crate::schema::types::{
    node::{IdData, NodeIdent},
    scalars::DateTimeScalar,
    user::{User, UserRegisterInput},
};

use super::{password_data::PasswordData, AuthMethodType};

#[derive(Error)]
pub enum RegistrationError {
    #[error(message = "Database error")]
    DbError(sqlx::Error),
    #[error(message = "User already exists")]
    UserAlreadyExists,
    #[error(message = "Password data not present")]
    PasswordDataNotPresent,
    #[error(message = "Password data invalid")]
    PasswordDataInvalid,
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
        id: IdData {
            ty: NodeIdent::User,
            uuid: user.id,
        }
        .to_id_scalar(),
        nickname: user.nickname,
        email: user.email,
        registered_at: DateTimeScalar(user.registered_at),
        deleted_at: user.deleted_at.map(DateTimeScalar),
    })
}
