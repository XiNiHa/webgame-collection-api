use std::collections::HashMap;

use sqlx::types::Uuid;
use tokio::sync::{mpsc, Mutex};

use crate::schema::types::{chat::Chat, scalars::UuidScalar};

pub struct ChatData {
    pub sender_id: Uuid,
    pub target_ids: Vec<Uuid>,
    pub message: String,
}

#[cfg(not(kds))]
lazy_static::lazy_static! {
    pub static ref CHANNEL_MAP: Mutex<HashMap<Uuid, mpsc::Sender<Chat>>> = Mutex::new(HashMap::new());
}

#[cfg(not(sqs))]
pub async fn broadcast(mut rx: mpsc::Receiver<ChatData>) {
    while let Some(chat) = rx.recv().await {
        let mut map_guard = CHANNEL_MAP.lock().await;
        for user_id in chat.target_ids {
            if let Some(tx) = map_guard.get(&user_id) {
                if tx.is_closed() {
                    map_guard.remove(&user_id);
                } else if let Err(e) = tx
                    .send(Chat {
                        sender_id: UuidScalar(chat.sender_id),
                        message: chat.message.clone(),
                    })
                    .await
                {
                    println!("{}", e)
                }
            }
        }
    }
}
