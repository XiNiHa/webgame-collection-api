use async_graphql::validators::Email;
use async_graphql::*;
use sqlx::PgPool;

use crate::{
    auth::{password_data::PasswordData, register::register, AuthMethodType},
    config::CONFIG,
    error::Error,
    schema::types::user::{User, UserRegisterInput},
};

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
}
