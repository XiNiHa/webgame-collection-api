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
