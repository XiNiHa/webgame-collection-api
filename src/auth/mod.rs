pub mod auth_info;
pub mod login;
pub mod password_data;
pub mod register;

#[derive(sqlx::Type, PartialEq)]
#[sqlx(type_name = "auth_method_type", rename_all = "lowercase")]
pub enum AuthMethodType {
    Email,
    Kakao,
    Google,
    Facebook,
}

static INV_AUTH_TOKEN_REDIS_KEY: &str = "auth/invalidated_auth_token:";

pub fn get_invalid_token_key(token: &str) -> String {
    let mut key = INV_AUTH_TOKEN_REDIS_KEY.to_owned();
    key.push_str(token);
    key
}
