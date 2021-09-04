use std::convert::TryFrom;

use async_graphql::*;
use sqlx::PgPool;

use crate::{
    error::Error,
    schema::types::node::{IdData, IdDataError, Node},
};

#[derive(Debug)]
enum NodeQueryError {
    IdParseError(IdDataError),
    ResolverNotFound,
}

impl Error for NodeQueryError {
    fn message(&self) -> String {
        match self {
            NodeQueryError::IdParseError(_) => "ID parsing failed",
            NodeQueryError::ResolverNotFound => "Resolver not found for the node type",
        }
        .to_owned()
    }

    fn code(&self) -> String {
        format!("NodeQueryError::{:?}", self)
    }
}

#[derive(Default)]
pub struct NodeQuery;

#[Object]
impl NodeQuery {
    async fn node(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Node>> {
        let id_data = IdData::try_from(id).map_err(|e| NodeQueryError::IdParseError(e).build())?;
        let pool = ctx.data::<PgPool>()?;

        let node =
            id_data
            .ty
            .resolve(&id_data.uuid, &pool)
            .await
            .ok_or(NodeQueryError::ResolverNotFound.build())?;

        Ok(node)
    }
}
