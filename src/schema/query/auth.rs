use async_graphql::*;
use async_graphql::validators::Email;

use crate::{
    auth::password_data::PasswordData,
    schema::{types::user::LoginResult, AppContext},
};

#[derive(Default)]
pub struct AuthQuery;

#[Object]
impl AuthQuery {
    async fn login_email(
        &self,
        ctx: &Context<'_>,
        #[graphql(validator(Email))]
        email: String,
        password: String,
    ) -> Result<Option<LoginResult>> {
        let AppContext { pool, .. } = ctx.data::<AppContext>()?;

        // TODO: move this to a separate function

        let method = sqlx::query!(
            r#"
            SELECT
                user_id,
                extra_info
            FROM public.user_auth_method
            WHERE type = 'email' AND identifier = $1
            "#,
            email
        )
        .fetch_optional(pool)
        .await?
        .ok_or(Error::new("No matching auth method found"))?;

        let password_data: PasswordData = serde_json::from_value(
            method
                .extra_info
                .ok_or(Error::new("Invalid auth method data detected"))?,
        )?;

        if password_data.verify(&password) {
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
