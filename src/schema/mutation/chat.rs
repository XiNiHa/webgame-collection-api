use std::convert::TryFrom;

use async_graphql::*;
use tokio::sync::mpsc::Sender;

use crate::{
    auth::auth_info::AuthInfo,
    chat::ChatData,
    error::Error,
    schema::types::{
        chat::Chat,
        node::{IdData, IdDataError, NodeIdent},
    },
};

#[derive(Debug)]
enum ChatMutationError {
    NotAuthorized,
    InvalidTargetId(IdDataError),
}

impl Error for ChatMutationError {
    fn message(&self) -> String {
        match self {
            ChatMutationError::NotAuthorized => "Not authorized",
            ChatMutationError::InvalidTargetId(_) => "Invalid target ID",
        }
        .to_owned()
    }

    fn code(&self) -> String {
        format!("ChatMutationError::{:?}", self)
    }
}

#[derive(Default)]
pub struct ChatMutation;

#[Object]
impl ChatMutation {
    async fn direct_message(
        &self,
        ctx: &Context<'_>,
        target_id: ID,
        message: String,
    ) -> Result<Chat> {
        let AuthInfo { user_id } = ctx.data::<AuthInfo>()?;
        let chat_tx = ctx.data::<Sender<ChatData>>()?;
        let sender_id = user_id.ok_or(ChatMutationError::NotAuthorized.build())?;
        let target_uuid = IdData::try_from(target_id)
            .map_err(|e| ChatMutationError::InvalidTargetId(e).build())?
            .uuid;

        chat_tx
            .send(ChatData {
                sender_id,
                target_ids: vec![target_uuid],
                message: message.clone(),
            })
            .await?;

        Ok(Chat {
            sender_id: IdData {
                ty: NodeIdent::User,
                uuid: sender_id,
            }
            .to_id_scalar(),
            message: message.clone(),
        })
    }
}
