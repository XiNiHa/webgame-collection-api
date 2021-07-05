pub mod register;
pub mod password_data;
pub mod login;

#[derive(sqlx::Type, PartialEq)]
#[sqlx(type_name = "auth_method_type", rename_all = "lowercase")]
pub enum AuthMethodType {
    Email,
    Kakao,
    Google,
    Facebook,
}
