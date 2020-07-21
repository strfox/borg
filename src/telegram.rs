use std::{error, fmt, sync::Arc};

use carapax::types::{Message};
use carapax::{
    longpoll::LongPoll, Api, ApiError, Dispatcher, ErrorPolicy,
    HandlerResult, LoggingErrorHandler,
};
use futures::lock::Mutex;

use crate::{
    borg::Borg,
    config,
    config::{BehaviorOverride, BehaviorOverrideValueResolver},
};
use carapax::handler;
use carapax::methods::SendMessage;
use futures::TryFutureExt;


/////////////////////////////////////////////////////////////////////////////
// RunError
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum RunError {
    SocketAddressParseError(SocketAddrParseError),
    LongPollError(LongPollError),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RunError::SocketAddressParseError(ref e) => e.fmt(f),
            RunError::LongPollError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for RunError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            RunError::SocketAddressParseError(ref e) => Some(e),
            RunError::LongPollError(ref e) => Some(e),
        }
    }
}

impl From<SocketAddrParseError> for RunError {
    fn from(err: SocketAddrParseError) -> RunError {
        RunError::SocketAddressParseError(err)
    }
}

impl From<LongPollError> for RunError {
    fn from(err: LongPollError) -> RunError {
        RunError::LongPollError(err)
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
// LongPoll Error
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LongPollError {
    message: String,
}

impl fmt::Display for LongPollError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for LongPollError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/////////////////////////////////////////////////////////////////////////////
// Context Struct
/////////////////////////////////////////////////////////////////////////////

pub struct Context {
    borg: Arc<Mutex<Borg>>,
    platform_config: config::TelegramPlatform,
    api: Api,
}

/////////////////////////////////////////////////////////////////////////////
// Context Implementations
/////////////////////////////////////////////////////////////////////////////

impl Context {
    pub fn new(
        platform_config: config::TelegramPlatform,
        borg: Arc<Mutex<Borg>>,
    ) -> Result<Context, ApiError> {
        let token = platform_config.token.clone();
        Api::new(token).map(|api| Context {
            borg,
            platform_config,
            api,
        })
    }

    fn behavior_for_chat(&self, chat_id: &i64) -> Option<BehaviorOverrideValueResolver> {
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

    fn override_for_chat(&self, chat_id: &i64) -> Option<&BehaviorOverride> {
        let chat_id: i64 = (*chat_id).into();
        let chat_id = chat_id.to_string();
        self.platform_config
            .chat_behaviors
            .as_ref()
            .and_then(|bs| bs.iter().find(|cb| cb.chat_id == chat_id))
            .map(|cb| &cb.behavior)
    }
}

/////////////////////////////////////////////////////////////////////////////
// Update Handler
/////////////////////////////////////////////////////////////////////////////

#[handler]
async fn handle(context: &Arc<Mutex<Context>>, message: Message) -> HandlerResult {
    let context = context.lock().await;
    if !message_is_older_than_now(&message) {
        if let (Some(text), Some(user)) = (message.get_text(), message.get_user()) {
            let behavior = context.behavior_for_chat(&message.get_chat_id());
            let input = text.data.as_str();
            let user_id = &user.id.to_string();
            let chat_id = message.get_chat_id();
            let mut borg = context.borg.lock().await;

            if borg.should_learn(user_id, input, &behavior) {
                borg.learn(input);
            }

            if borg.should_reply_to(user_id, input, &behavior) {
                if let Some(response) = borg.respond_to(input) {
                    match context
                        .api
                        .execute(SendMessage::new(chat_id, response))
                        .await {
                        Ok(..) => {}
                        Err(e) => {
                            error!("ExecuteError: {}", e);
                        }
                    }
                }
            }
        }
    }
    HandlerResult::Continue
}

/////////////////////////////////////////////////////////////////////////////
// Utility Functions
/////////////////////////////////////////////////////////////////////////////

fn message_is_older_than_now(message: &Message) -> bool {
    message.date < crate::util::unix_time() as i64
}

/////////////////////////////////////////////////////////////////////////////
// Run Method
/////////////////////////////////////////////////////////////////////////////

pub async fn run(context: Arc<Mutex<Context>>) -> Result<(), RunError> {
    let mut dispatcher = Dispatcher::new(context.clone());
    dispatcher.set_error_handler(LoggingErrorHandler::new(ErrorPolicy::Continue));
    dispatcher.add_handler(handle);

    let context = context.lock().await.api.clone();

    LongPoll::new(context, dispatcher).run().await;
    Ok(())
}
