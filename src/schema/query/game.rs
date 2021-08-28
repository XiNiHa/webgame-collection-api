use async_graphql::{connection::*, *};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::schema::types::{
    game::Game, localized_string::LocalizedString, scalars::DateTimeScalar,
};

#[derive(Default)]
pub struct GameQuery;

#[Object]
impl GameQuery {
    async fn games(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<DateTimeScalar, Game>> {
        let pool = ctx.data::<PgPool>()?;

        query(
            after,
            before,
            first,
            last,
            |after: Option<DateTimeScalar>, before, first, last| async move {
                let after = after.map(|scalar| scalar.0);
                let before = before.map(|scalar| scalar.0);

                let rows = sqlx::query!(
                    r#"
                    SELECT
                        id,
                        name AS "name: LocalizedString",
                        min_players,
                        max_players,
                        description AS "description: LocalizedString",
                        created_at,
                        after AS "after: DateTime<Utc>",
                        before AS "before: DateTime<Utc>"
                    FROM
                        public.game,
                        (VALUES ($1::TIMESTAMPTZ, $2::TIMESTAMPTZ)) s(after, before)
                    WHERE
                        (after IS NULL OR created_at > after) AND
                        (before IS NULL OR created_at < before)
                    ORDER BY
                        (CASE WHEN $3 THEN created_at END) DESC,
                        created_at ASC
                    LIMIT $4 + 1
                    "#,
                    after,
                    before,
                    last.is_some(),
                    first.or(last).unwrap_or(10) as i32,
                )
                .fetch_all(pool)
                .await?;

                let mut connection = Connection::new(
                    first.is_none() && rows.len() > last.unwrap_or(10),
                    last.is_none() && rows.len() > first.unwrap_or(10),
                );
                let iter = rows.into_iter().map(|row| {
                    Edge::new(
                        DateTimeScalar(row.created_at),
                        Game {
                            id: row.id,
                            name: row.name,
                            min_players: row.min_players,
                            max_players: row.max_players,
                            description: row.description,
                        },
                    )
                });
                if last.is_some() {
                    connection.append(iter.take(last.unwrap_or(10)).rev());
                } else {
                    connection.append(iter.take(first.unwrap_or(10)));
                }
                Ok(connection)
            },
        )
        .await
    }
}
