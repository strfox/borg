use crate::{
    config,
    config::{BehaviorOverride, BehaviorOverrideValueResolver},
    seeborg::SeeBorg,
};
use carapax::{handler, types::Command, webhook, Api, ApiError, Dispatcher};
use futures::lock::Mutex;
use std::{error, fmt, net::SocketAddr, num::ParseIntError, sync::Arc};

/////////////////////////////////////////////////////////////////////////////
// RunError
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum RunError {
    SocketAddressParseError(SocketAddrParseError),
    WebhookError(WebhookError),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RunError::SocketAddressParseError(ref e) => e.fmt(f),
            RunError::WebhookError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for RunError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            RunError::SocketAddressParseError(ref e) => Some(e),
            RunError::WebhookError(ref e) => Some(e),
        }
    }
}

impl From<SocketAddrParseError> for RunError {
    fn from(err: SocketAddrParseError) -> RunError {
        RunError::SocketAddressParseError(err)
    }
}

impl From<WebhookError> for RunError {
    fn from(err: WebhookError) -> RunError {
        RunError::WebhookError(err)
    }
}

/////////////////////////////////////////////////////////////////////////////
// SocketAddrParse Error
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct SocketAddrParseError {
    bad_string: String,
}

impl fmt::Display for SocketAddrParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cannot parse socket address: {}", self.bad_string)
    }
}

impl error::Error for SocketAddrParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/////////////////////////////////////////////////////////////////////////////
// Webhook Error
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct WebhookError {
    message: String,
}

impl fmt::Display for WebhookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Webhook error: {}", self.message)
    }
}

impl error::Error for WebhookError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/////////////////////////////////////////////////////////////////////////////
// Telegram Struct
/////////////////////////////////////////////////////////////////////////////

pub struct Telegram {
    seeborg: Arc<Mutex<SeeBorg>>,
    platform_config: config::TelegramPlatform,
    api: Api,
}

/////////////////////////////////////////////////////////////////////////////
// Telegram Implementations
/////////////////////////////////////////////////////////////////////////////

impl Telegram {
    pub fn new(
        platform_config: config::TelegramPlatform,
        seeborg: Arc<Mutex<SeeBorg>>,
    ) -> Result<Telegram, ApiError> {
        let token = platform_config.token.clone();
        Api::new(token).map(|api| Telegram {
            seeborg,
            platform_config,
            api,
        })
    }

    pub async fn run(&mut self) -> Result<(), RunError> {
        let mut dispatcher = Dispatcher::new(self.api.clone());

        #[handler(command = "/start")]
        async fn start_command_handler(_context: &Api, _command: Command) {
            todo!();
        }

        dispatcher.add_handler(start_command_handler);

        let socket_addr: SocketAddr = match self.platform_config.webhook_bind_address.parse() {
            Ok(o) => o,
            Err(e) => {
                return Err(RunError::SocketAddressParseError(SocketAddrParseError {
                    bad_string: self.platform_config.webhook_bind_address.to_string(),
                }))
            }
        };

        match webhook::run_server(socket_addr, "/seeborg", dispatcher).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RunError::WebhookError(WebhookError {
                message: e.to_string(),
            })),
        }
    }
    /*
    fn behavior_for_chat<'a>(
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
    }*/
}
/*
fn message_is_older_than_now(message: &Message) -> bool {
    message.date < crate::util::unix_time() as i64
}*/
