use sqlx::PgPool;
use uuid::Uuid;

use crate::schema::types::{
    game::Game,
    localized_string::LocalizedString,
    node::{IdData, Node, NodeIdent},
};

pub async fn game_resolver(uuid: &Uuid, pool: &PgPool) -> Option<Node> {
    let game = sqlx::query!(
        r#"
        SELECT
            id,
            name AS "name: LocalizedString",
            min_players,
            max_players,
            description AS "description: LocalizedString",
            created_at
        FROM public.game
        WHERE id = $1
        "#,
        uuid
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    Some(Node::Game(Game {
        id: IdData {
            ty: NodeIdent::Game,
            uuid: game.id,
        }
        .to_id_scalar(),
        name: game.name,
        min_players: game.min_players,
        max_players: game.max_players,
        description: game.description,
    }))
}
