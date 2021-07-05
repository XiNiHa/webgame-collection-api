use std::iter::FromIterator;

use async_graphql::*;

use crate::schema::{
    types::{game::Game, localized_string::LocalizedString},
    AppContext,
};

#[derive(Default)]
pub struct GameQuery;

#[Object]
impl GameQuery {
    async fn games(&self, ctx: &Context<'_>) -> Result<Vec<Game>> {
        let AppContext { pool, .. } = ctx.data::<AppContext>()?;

        let rows = sqlx::query!(
            r#"
            SELECT
                id,
                name AS "name: LocalizedString",
                min_players,
                max_players,
                description AS "description: LocalizedString"
            FROM public.game
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(Vec::from_iter(rows.into_iter().map(|row| Game {
            id: row.id,
            name: row.name,
            min_players: row.min_players,
            max_players: row.max_players,
            description: row.description,
        })))
    }
}
