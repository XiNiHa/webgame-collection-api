use async_graphql::validators::Email;
use async_graphql::*;
use sqlx::PgPool;
use webgame_collection_api_macros::Error;

use crate::{
    auth::{
        auth_info::{AuthError, AuthInfo},
        login::{create_login_result, register_refresh_token, verify_auth_method},
        password_data::PasswordData,
        register::register,
        AuthMethodType,
    },
    config::CONFIG,
    error::Error,
    schema::types::user::{LoginResult, User, UserRegisterInput},
};

#[derive(Error)]
pub enum AuthMutationError {
    #[error(message = "Redis error")]
    RedisError(redis::RedisError),
}

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
    async fn register_email(
        &self,
        ctx: &Context<'_>,
        input: UserRegisterInput,
        #[graphql(validator(Email))] email: String,
        password: String,
    ) -> Result<User> {
        let pool = ctx.data::<PgPool>()?;

        let password_data =
            PasswordData::new(&password, CONFIG.pbkdf2_salt_size, CONFIG.pbkdf2_iterations)
                .map_err(|e| e.build())?;

        register(
            pool,
            input,
            AuthMethodType::Email,
            email,
            Some(password_data),
        )
        .await
        .map_err(|e| e.build())
    }

    async fn login_email(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(Email))] email: String,
        password: String,
    ) -> Result<Option<LoginResult>> {
        let pg_pool = ctx.data::<PgPool>()?;
        let mut redis_conn = ctx.data::<deadpool_redis::Pool>()?.get().await?;

        let user_id = verify_auth_method(pg_pool, AuthMethodType::Email, email, Some(password))
            .await
            .map_err(|e| e.build())?;

        let login_result =
            create_login_result(&user_id, &CONFIG.jwt_secret, CONFIG.refresh_token_size)
                .map_err(|e| e.build())?;

        Ok(Some(login_result))
    }

    async fn logout(&self, ctx: &Context<'_>) -> Result<bool> {
        let auth_info = ctx
            .data::<Option<AuthInfo>>()?
            .as_ref()
            .ok_or(AuthError::Invalidated.build())?;
        let mut redis_conn = ctx.data::<deadpool_redis::Pool>()?.get().await?;

        auth_info
            .invalidate(&mut redis_conn)
            .await
            .map_err(|e| AuthMutationError::RedisError(e).build())
    }
}
