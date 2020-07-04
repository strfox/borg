use crate::seeborg::SeeBorg;
use crate::PlatformError;
use futures::lock::Mutex;
use futures::StreamExt;
use std::sync::Arc;
use telegram_bot::requests::send_message::CanReplySendMessage;
use telegram_bot::{Api, MessageKind, UpdateKind};

pub struct Telegram {
    seeborg: Arc<Mutex<SeeBorg>>,
    api: Api,
}

impl Telegram {
    pub async fn new(seeborg: Arc<Mutex<SeeBorg>>) -> Telegram {
        let token = seeborg
            .lock()
            .await
            .config
            .telegram
            .as_ref()
            .expect("Telegram not defined")
            .token
            .clone();
        Telegram {
            seeborg: seeborg,
            api: Api::new(token),
        }
    }
    pub async fn poll(&mut self) -> Result<(), PlatformError> {
        let mut stream = self.api.stream();
        while let Some(update) = stream.next().await {
            let update = update?;
            if let UpdateKind::Message(message) = update.kind {
                if let MessageKind::Text { ref data, .. } = message.kind {
                    match self.seeborg.lock().await.respond_to(data) {
                        Some(response) => {
                            self.api.send(message.text_reply(response)).await?;
                        }
                        None => {}
                    }
                }
            }
        }
        Ok(())
    }
}
