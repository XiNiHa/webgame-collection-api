use async_graphql::*;

use crate::{
    auth::password_data::PasswordData,
    schema::{
        types::{
            scalars::{DateTimeScalar, UuidScalar},
            user::{User, UserRegisterInput},
        },
        AppContext,
    },
};

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
    // TODO: add input validation
    async fn register_email(
        &self,
        ctx: &Context<'_>,
        input: UserRegisterInput,
        email: String,
        password: String,
    ) -> Result<User> {
        let AppContext { pool, config } = ctx.data::<AppContext>()?;

        // TODO: move this to a separate function

        // TODO: check if user already exists
        let user = sqlx::query!(
            r#"
            INSERT INTO public.user (id, nickname, email, registered_at)
            VALUES (uuid_generate_v4(), $1, $2, CURRENT_TIMESTAMP)
            RETURNING *
            "#,
            input.nickname,
            input.email
        )
        .fetch_one(pool)
        .await?;

        let password_data =
            PasswordData::new(&password, config.pbkdf2_salt_size, config.pbkdf2_iterations)
                .map_err(|_e| Error::new("Password encryption failed"))?;

        // TODO: check if the method already exists
        let _auth_result = sqlx::query!(
            r#"
            INSERT INTO public.user_auth_method (user_id, type, identifier, extra_info)
            VALUES ($1, 'EMAIL', $2, $3)
            "#,
            user.id,
            email,
            serde_json::to_value(password_data)?
        )
        .execute(pool)
        .await?;

        Ok(User {
            id: UuidScalar(user.id),
            nickname: user.nickname,
            email: user.email,
            registered_at: DateTimeScalar(user.registered_at),
            deleted_at: user.deleted_at.map(DateTimeScalar),
        })
    }
}
