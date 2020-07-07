use crate::{
    config,
    config::{BehaviorOverride, BehaviorOverrideValueResolver},
    seeborg::SeeBorg,
    PlatformError,
};
use futures::lock::Mutex;
use futures::StreamExt;
use std::sync::Arc;
use telegram_bot::{
    requests::send_message::CanSendMessage, types::Message, Api, ChatId, MessageKind, UpdateKind,
};

pub struct Telegram {
    seeborg: Arc<Mutex<SeeBorg>>,
    platform_config: config::Platform,
    api: Api,
}

impl Telegram {
    pub fn new(platform_config: config::Platform, seeborg: Arc<Mutex<SeeBorg>>) -> Telegram {
        let token = platform_config.token.clone();
        Telegram {
            seeborg,
            platform_config,
            api: Api::new(token),
        }
    }

    pub async fn poll(&mut self) -> Result<(), PlatformError> {
        let mut stream = self.api.stream();
        while let Some(update) = stream.next().await {
            let update = update?;
            if let UpdateKind::Message(message) = update.kind {
                if message_is_older_than_now(&message) {
                    continue;
                }
                if let MessageKind::Text { ref data, .. } = message.kind {
                    let mut seeborg = self.seeborg.lock().await;
                    let chat_id: i64 = message.chat.id().into();
                    if let Some(response) = seeborg.respond_to(data) {
                        self.api.send(message.chat.text(response)).await?;
                    }
                    seeborg.learn(data);
                }
            }
        }
        Ok(())
    }

    async fn behavior_for_chat<'a>(
        &'a self,
        chat_id: &ChatId,
    ) -> Option<BehaviorOverrideValueResolver<'a>> {
        self.platform_config
            .behavior
            .as_ref()
            .map(|b| {
                (
                    b,
                    self.override_for_chat(&chat_id)
                        .map(|o| Box::new(BehaviorOverrideValueResolver::new(o, None))),
                )
            })
            .map(|(b, o)| BehaviorOverrideValueResolver::new(b, o))
    }

    fn override_for_chat(&self, chat_id: &ChatId) -> Option<&BehaviorOverride> {
        let chat_id: i64 = (*chat_id).into();
        let chat_id = chat_id.to_string();
        self.platform_config
            .chat_behaviors
            .as_ref()
            .and_then(|bs| bs.iter().find(|cb| cb.chat_id == chat_id))
            .map(|cb| &cb.behavior)
    }
}

fn message_is_older_than_now(message: &Message) -> bool {
    message.date < crate::util::unix_time() as i64
}
