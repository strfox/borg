use crate::seeborg::SeeBorg;
use futures::StreamExt;
use telegram_bot::requests::send_message::CanReplySendMessage;
use telegram_bot::{Api, MessageKind, UpdateKind};

pub struct Telegram<'a> {
    seeborg: &'a mut SeeBorg,
    api: Api,
}

impl Telegram<'_> {
    pub fn new<'a>(seeborg: &'a mut SeeBorg) -> Telegram<'a> {
        let token = seeborg
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

    pub async fn poll(&mut self) -> Result<(), telegram_bot::Error> {
        let mut stream = self.api.stream();
        while let Some(update) = stream.next().await {
            let update = update?;
            if let UpdateKind::Message(message) = update.kind {
                if let MessageKind::Text { ref data, .. } = message.kind {
                    match self.seeborg.respond_to(data) {
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
