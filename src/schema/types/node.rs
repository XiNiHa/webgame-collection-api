use std::{convert::TryFrom, iter::FromIterator};

use async_graphql::*;
use uuid::Uuid;
use webgame_collection_api_macros::GenNodeIdent;

use super::{
    game::Game,
    resolvers::{game::game_resolver, user::user_resolver},
    user::User,
};

#[derive(Interface, GenNodeIdent)]
#[graphql(field(name = "id", type = "&ID"))]
pub enum Node {
    #[node_ident(resolver = "user_resolver")]
    User(User),
    #[node_ident(resolver = "game_resolver")]
    Game(Game),
}

pub struct IdData {
    pub ty: NodeIdent,
    pub uuid: Uuid,
}

#[derive(Debug)]
pub enum IdDataError {
    Base64Error(base64::DecodeError),
    StringifyError(std::string::FromUtf8Error),
    InvalidFormat,
    InvalidType(Box<String>),
    UuidError(sqlx::types::uuid::Error),
}

impl IdData {
    pub fn to_id_scalar(&self) -> ID {
        ID(base64::encode(format!(
            "{}:{}",
            self.ty.to_string(),
            self.uuid.to_string()
        )))
    }
}

impl TryFrom<ID> for IdData {
    type Error = IdDataError;

    fn try_from(value: ID) -> Result<Self, Self::Error> {
        let decoded = String::from_utf8(base64::decode(value.0).map_err(IdDataError::Base64Error)?)
            .map_err(IdDataError::StringifyError)?;

        let splitted = Vec::from_iter(decoded.split(':').into_iter());
        if splitted.len() != 2 {
            return Err(IdDataError::InvalidFormat);
        }

        let mut splitted_iter = splitted.into_iter();

        let type_str = splitted_iter.next().unwrap();
        let uuid_str = splitted_iter.next().unwrap();

        let ty = NodeIdent::from_str(type_str)
            .ok_or(IdDataError::InvalidType(Box::new(type_str.to_string())))?;
        let uuid = Uuid::parse_str(uuid_str).map_err(IdDataError::UuidError)?;

        Ok(IdData { ty, uuid })
    }
}
