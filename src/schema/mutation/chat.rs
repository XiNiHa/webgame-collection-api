use async_graphql::*;

use crate::{
    auth::auth_info::AuthInfo,
    chat::ChatData,
    error::Error,
    schema::types::{chat::Chat, scalars::UuidScalar},
};

#[derive(Debug)]
enum ChatMutationError {
    NotAuthorized,
}

impl Error for ChatMutationError {
    fn message(&self) -> String {
        match self {
            ChatMutationError::NotAuthorized => "Not authorized",
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
        target_id: UuidScalar,
        message: String,
    ) -> Result<Chat> {
        let AuthInfo { user_id } = ctx.data::<AuthInfo>()?;
        let sender_id = user_id.ok_or(ChatMutationError::NotAuthorized.build())?;

        let mut buffer = crate::chat::CHAT_BUFFER.lock().await;

        buffer.push(ChatData {
            sender_id,
            target_ids: vec![target_id.0],
            message: message.clone(),
        });

        Ok(Chat {
            sender_id: UuidScalar(sender_id),
            message: message.clone(),
        })
    }
}
