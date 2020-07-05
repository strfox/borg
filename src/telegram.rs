use crate::seeborg::SeeBorg;
use crate::PlatformError;
use futures::lock::Mutex;
use futures::StreamExt;
use std::sync::Arc;
use telegram_bot::requests::send_message::{CanSendMessage};
use telegram_bot::{Api, MessageKind, UpdateKind};

pub struct Telegram {
    seeborg: Arc<Mutex<SeeBorg>>,
    api: Api,
}

impl Telegram {
    pub fn new(token: &str, seeborg: Arc<Mutex<SeeBorg>>) -> Telegram {
        Telegram {
            seeborg,
            api: Api::new(token),
        }
    }
    
    pub async fn poll(&mut self) -> Result<(), PlatformError> {
        let mut stream = self.api.stream();
        while let Some(update) = stream.next().await {
            let update = update?;
            if let UpdateKind::Message(message) = update.kind {
                if let MessageKind::Text { ref data, .. } = message.kind {
                    let mut seeborg = self.seeborg.lock().await;
                    if let Some(response) = seeborg.respond_to(data) {
                        self.api.send(message.chat.text(response)).await?;
                    }
                    seeborg.learn(data);
                }
            }
        }
        Ok(())
    }
}
