use sqlx::PgPool;
use uuid::Uuid;

use crate::schema::types::{
    node::{IdData, Node, NodeIdent},
    scalars::DateTimeScalar,
    user::User,
};

pub async fn user_resolver(uuid: &Uuid, pool: &PgPool) -> Option<Node> {
    let user = sqlx::query!(
        r#"
        SELECT * FROM public.user
        WHERE id = $1
        "#,
        uuid,
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    Some(Node::User(User {
        id: IdData {
            ty: NodeIdent::User,
            uuid: user.id,
        }
        .to_id_scalar(),
        nickname: user.nickname,
        email: user.email,
        registered_at: DateTimeScalar(user.registered_at),
        deleted_at: user.deleted_at.map(DateTimeScalar),
    }))
}
