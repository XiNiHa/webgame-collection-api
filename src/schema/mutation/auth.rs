use async_graphql::validators::Email;
use async_graphql::*;
use sqlx::PgPool;
use webgame_collection_api_macros::Error;

use crate::{
    auth::{
        auth_info::AuthInfo,
        login::{create_access_token, verify_auth_method},
        password_data::PasswordData,
        refresh::{
            check_refresh, create_refresh_token, register_refresh_token, RefreshCheckResult,
        },
        register::register,
        AuthMethodType,
    },
    config::CONFIG,
    error::Error,
    schema::types::user::{LoginResult, RefreshResult, User, UserRegisterInput},
};

#[derive(Error)]
pub enum AuthMutationError {
    #[error(message = "Redis error")]
    RedisError(redis::RedisError),
    #[error(message = "Token creation failed")]
    TokenCreationFailed,
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

        let access_token = create_access_token(&user_id, &CONFIG.jwt_secret)
            .ok_or(AuthMutationError::TokenCreationFailed.build())?;
        let refresh_token = create_refresh_token(CONFIG.refresh_token_size)
            .ok_or(AuthMutationError::TokenCreationFailed.build())?;

        register_refresh_token(&refresh_token, &mut redis_conn)
            .await
            .map_err(|e| AuthMutationError::RedisError(e).build())?;

        Ok(Some(LoginResult {
            access_token,
            refresh_token,
        }))
    }

    async fn logout(&self, ctx: &Context<'_>) -> Result<bool> {
        let auth_info = ctx.data::<AuthInfo>()?;
        let mut redis_conn = ctx.data::<deadpool_redis::Pool>()?.get().await?;

        auth_info
            .invalidate(&mut redis_conn)
            .await
            .map_err(|e| AuthMutationError::RedisError(e).build())
    }

    async fn refresh_auth(
        &self,
        ctx: &Context<'_>,
        refresh_token: String,
    ) -> Result<Option<RefreshResult>> {
        let auth_info = ctx.data::<AuthInfo>()?;
        let mut redis_conn = ctx.data::<deadpool_redis::Pool>()?.get().await?;
        let user_id = auth_info.get_user_id().map_err(|e| e.build())?;

        let refresh_check_result = check_refresh(&refresh_token, &mut redis_conn)
            .await
            .map_err(|e| AuthMutationError::RedisError(e).build())?;

        let access_token = match refresh_check_result {
            RefreshCheckResult::Both | RefreshCheckResult::OnlyAccessToken => {
                let token = create_access_token(&user_id, &CONFIG.jwt_secret)
                    .ok_or(AuthMutationError::TokenCreationFailed.build())?;

                auth_info
                    .invalidate(&mut redis_conn)
                    .await
                    .map_err(|e| AuthMutationError::RedisError(e).build())?;

                Some(token)
            }
            _ => None,
        };
        let refresh_token = match refresh_check_result {
            RefreshCheckResult::Both => Some(
                create_refresh_token(CONFIG.refresh_token_size)
                    .ok_or(AuthMutationError::TokenCreationFailed.build())?,
            ),
            _ => None,
        };

        match access_token {
            Some(access_token) => Ok(Some(RefreshResult {
                access_token,
                refresh_token,
            })),
            None => Ok(None),
        }
    }
}
