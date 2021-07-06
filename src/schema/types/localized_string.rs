use async_graphql::*;

#[derive(sqlx::Type, Debug, SimpleObject, Clone)]
#[sqlx(type_name = "localized_string")]
pub struct LocalizedString {
    pub ko: String,
    pub en: String,
}
