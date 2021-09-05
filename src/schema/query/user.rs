use async_graphql::*;
use sqlx::PgPool;

use crate::{
    auth::auth_info::AuthInfo,
    error::Error,
    schema::types::{
        node::{IdData, NodeIdent},
        scalars::DateTimeScalar,
        user::User,
    },
};

#[derive(Debug)]
enum UserQueryError {
    DbError(sqlx::Error),
    NotFound,
}

impl Error for UserQueryError {
    fn message(&self) -> String {
        match self {
            UserQueryError::DbError(_) => "Database error",
            UserQueryError::NotFound => "Not found",
        }
        .to_owned()
    }

    fn code(&self) -> String {
        format!("UserQueryError::{:?}", self)
    }
}

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let pool = ctx.data::<PgPool>()?;
        let auth_info = ctx.data::<AuthInfo>()?;
        let user_id = auth_info.get_user_id().map_err(|e| e.build())?;

        let user = sqlx::query!(
            r#"
            SELECT * FROM public.user
            WHERE id = $1
            "#,
            user_id,
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| UserQueryError::DbError(e).build())?
        .ok_or(UserQueryError::NotFound.build())?;

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
}
