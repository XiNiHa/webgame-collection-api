use super::scalars::DateTimeScalar;
use async_graphql::validators::Email;
use async_graphql::*;

#[derive(SimpleObject)]
pub struct User {
    pub id: ID,
    pub nickname: String,
    pub email: String,
    pub registered_at: DateTimeScalar,
    pub deleted_at: Option<DateTimeScalar>,
}

#[derive(InputObject)]
pub struct UserRegisterInput {
    pub nickname: String,
    #[graphql(validator(Email))]
    pub email: String,
}

#[derive(SimpleObject)]
pub struct LoginResult {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(SimpleObject)]
pub struct RefreshResult {
    pub access_token: String,
    pub refresh_token: Option<String>,
}
