use async_graphql::validators::Email;
use async_graphql::*;
use sqlx::PgPool;

use crate::{
    auth::{
        login::{create_login_result, verify_auth_method},
        AuthMethodType,
    },
    config::CONFIG,
    error::Error,
    schema::types::user::LoginResult,
};

#[derive(Default)]
pub struct AuthQuery;

#[Object]
impl AuthQuery {
    async fn login_email(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(Email))] email: String,
        password: String,
    ) -> Result<Option<LoginResult>> {
        let pool = ctx.data::<PgPool>()?;

        let user_id = verify_auth_method(pool, AuthMethodType::Email, email, Some(password))
            .await
            .map_err(|e| e.build())?;

        let login_result =
            create_login_result(&user_id, &CONFIG.jwt_secret, CONFIG.refresh_token_size)
                .map_err(|e| e.build())?;

        Ok(Some(login_result))
    }
}
