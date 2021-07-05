use async_graphql::validators::Email;
use async_graphql::*;

use crate::{
    auth::{password_data::PasswordData, register::register, AuthMethodType},
    error::Error,
    schema::{
        types::user::{User, UserRegisterInput},
        AppContext,
    },
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
        let AppContext { pool, config } = ctx.data::<AppContext>()?;

        let password_data =
            PasswordData::new(&password, config.pbkdf2_salt_size, config.pbkdf2_iterations)
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
