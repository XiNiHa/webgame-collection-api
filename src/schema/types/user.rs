use super::scalars::{DateTimeScalar, UuidScalar};
use async_graphql::*;

#[derive(SimpleObject)]
pub struct User {
    pub id: UuidScalar,
    pub nickname: String,
    pub email: String,
    pub registered_at: DateTimeScalar,
    pub deleted_at: Option<DateTimeScalar>,
}

#[derive(InputObject)]
pub struct UserRegisterInput {
    pub nickname: String,
    pub email: String,
}

#[derive(SimpleObject)]
pub struct LoginResult {
    pub access_token: String,
    pub refresh_token: String,
}
