use async_graphql::validators::Email;
use async_graphql::*;

use crate::{auth::{login::verify_auth_method, AuthMethodType}, error::Error, schema::{types::user::LoginResult, AppContext}};

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
        let AppContext { pool, .. } = ctx.data::<AppContext>()?;

        if verify_auth_method(pool, AuthMethodType::Email, email, Some(password))
            .await
            .map_err(|e| e.build())?
        {
            // TODO: generate actual tokens and return it
            Ok(Some(LoginResult {
                access_token: "random access token".to_string(),
                refresh_token: "random refresh token".to_string(),
            }))
        } else {
            Ok(None)
        }
    }
}
