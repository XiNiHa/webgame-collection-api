use std::convert::TryFrom;

use async_graphql::*;
use sqlx::PgPool;
use webgame_collection_api_macros::Error;

use crate::{
    error::Error,
    schema::types::node::{IdData, IdDataError, Node},
};

#[derive(Error)]
enum NodeQueryError {
    #[error(message = "ID parsing failed")]
    IdParseError(IdDataError),
    #[error(message = "Resolver not found for the node type")]
    ResolverNotFound,
}

#[derive(Default)]
pub struct NodeQuery;

#[Object]
impl NodeQuery {
    async fn node(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Node>> {
        let id_data = IdData::try_from(id).map_err(|e| NodeQueryError::IdParseError(e).build())?;
        let pool = ctx.data::<PgPool>()?;

        let node = id_data
            .ty
            .resolve(&id_data.uuid, &pool)
            .await
            .ok_or(NodeQueryError::ResolverNotFound.build())?;

        Ok(node)
    }
}
