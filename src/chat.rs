use std::{collections::HashMap, iter::FromIterator, time::Duration};

use sqlx::types::Uuid;
use tokio::{
    sync::{mpsc::Sender, Mutex},
    time::sleep,
};

use crate::schema::types::{chat::Chat, scalars::UuidScalar};

pub struct ChatData {
    pub sender_id: Uuid,
    pub target_ids: Vec<Uuid>,
    pub message: String,
}

#[cfg(not(kds))]
lazy_static::lazy_static! {
    pub static ref CHAT_BUFFER: Mutex<Vec<ChatData>> = Mutex::new(Vec::new());
    pub static ref CHANNEL_MAP: Mutex<HashMap<Uuid, Sender<Chat>>> = Mutex::new(HashMap::new());
}

pub async fn broadcast() {
    loop {
        #[cfg(not(kds))]
        let chats = if cfg!(kds) {
            // TODO: implement kds feature
            Vec::new()
        } else {
            let mut buffer = CHAT_BUFFER.lock().await;
            Vec::from_iter(buffer.drain(0..))
        };

        for chat in chats {
            println!("<{}>: {}", chat.sender_id, chat.message);

            let mut map_guard = CHANNEL_MAP.lock().await;
            for user_id in chat.target_ids {
                let keys = map_guard.keys();
                println!("{:?}", keys);
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

        sleep(Duration::from_millis(100)).await;
    }
}
