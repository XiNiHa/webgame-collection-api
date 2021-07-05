use super::localized_string::LocalizedString;
use async_graphql::*;

#[derive(SimpleObject)]
pub struct Game {
    pub id: i32,
    pub name: LocalizedString,
    pub min_players: i16,
    pub max_players: i16,
    pub description: LocalizedString,
}
