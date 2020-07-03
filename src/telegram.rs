use crate::seeborg::SeeBorg;
use futures::StreamExt;
use telegram_bot::{Api, UpdateKind, MessageKind};
use telegram_bot::requests::send_message::CanReplySendMessage;

pub struct Telegram<'a> {
    seeborg: &'a SeeBorg,
    api: Api,
}

impl Telegram<'_> {
    pub fn new<'a>(seeborg: &'a SeeBorg) -> Telegram<'a> {
        Telegram {
            seeborg: seeborg,
            api: Api::new(
                &seeborg
                    .config
                    .telegram
                    .as_ref()
                    .expect("Telegram not defined")
                    .token,
            ),
        }
    }

    pub async fn poll(&self) -> Result<(), telegram_bot::Error> {
        let mut stream = self.api.stream();
        while let Some(update) = stream.next().await {
            let update = update?;
            if let UpdateKind::Message(message) = update.kind {
                if let MessageKind::Text { ref data, .. } = message.kind {
                    self.api.send(message.text_reply(format!(
                        "Hi, {}! You just wrote '{}'",
                        &message.from.first_name, data
                    )))
                    .await?;
                }
            }
        }
        Ok(())
    }
}
