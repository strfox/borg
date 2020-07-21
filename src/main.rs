mod pattern;

#[macro_use]
extern crate lazy_static;
extern crate futures;
extern crate onig;
extern crate rand_core;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate tokio;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
mod util;
mod borg;
mod config;
mod dictionary;
mod discord;
mod telegram;

use borg::Borg;
use config::{Config, ConfigError};
use dictionary::Dictionary;
use futures::lock::Mutex;
use futures::Future;
use std::error;
use std::fmt;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

/////////////////////////////////////////////////////////////////////////////
// Platform Error
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum PlatformError {
    TelegramError(telegram::RunError),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PlatformError::TelegramError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for PlatformError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            PlatformError::TelegramError(ref e) => Some(e),
        }
    }
}

impl From<telegram::RunError> for PlatformError {
    fn from(err: telegram::RunError) -> PlatformError {
        PlatformError::TelegramError(err)
    }
}

/////////////////////////////////////////////////////////////////////////////
// Constants
/////////////////////////////////////////////////////////////////////////////

const CONFIG_PATH: &str = "config.yml";

/////////////////////////////////////////////////////////////////////////////
// Types
/////////////////////////////////////////////////////////////////////////////

type PlatformTasks = Vec<Pin<Box<dyn Future<Output = Result<(), PlatformError>>>>>;

/////////////////////////////////////////////////////////////////////////////
// Main Function
/////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() {
    println!("Borg is here.");

    env_logger::init();

    let config = match Config::load(Path::new(CONFIG_PATH)) {
        Ok(c) => c,
        Err(e) => match e {
            ConfigError::IOError(e) => {
                error!(
                    "An I/O error happened and the program could not \
                    read the configuration file. Please make sure that the \
                    file exists and that the program has permissions to read \
                    it. Details: {:?}",
                    e
                );
                return;
            }
            ConfigError::YAMLError(e) => {
                error!(
                    "A YAML parsing error occurred. This is most \
                    likely due to a malformed configuration file. Please check \
                    that your configuration is correct and try again. \
                    Details on the YAML parsing error: {:?}",
                    e
                );
                return;
            }
        },
    };

    debug!("Config {:?} loaded.", CONFIG_PATH);

    let mut dict = match Dictionary::load(Path::new(&config.dictionary_path)) {
        Ok(d) => d,
        Err(e) => match e {
            dictionary::Error::IOError(e) => {
                error!(
                    "An I/O error happened while trying to read the dictionary \
                file, located at \"{:?}\". Please ensure that the file is present \
                at such location and make sure that this program has read and write \
                permissions. Details: {:?}",
                    config.dictionary_path, e
                );
                return;
            }
            dictionary::Error::JSONError(e) => {
                error!(
                    "A JSON parsing error occurred. This is most likely due to \
                a corrupted dictionary file. Please check the dictionary file for any \
                anomalies. Details on the JSON parsing error: {:?}",
                    e
                );
                return;
            }
        },
    };

    debug!("Dictionary {:?} loaded.", &config.dictionary_path);

    if dict.needs_to_build_indices() {
        warn!("Indices need to be built. Building indices.");
        dict.rebuild_indices();
        warn!("Indices built.");

        if let Err(e) = save_dictionary(&config, &dict) {
            error!("Couldn't save dictionary, error: {:?}", e)
        }
    }

    let borg = Arc::new(Mutex::new(Borg::new(dict, config.behavior)));
    let mut tasks: PlatformTasks = vec![];

    let telegram_context = match config.telegram {
        Some(telegram_config) => Some(Arc::new(Mutex::new(
            match telegram::Context::new(telegram_config, borg.clone()) {
                Ok(o) => o,
                Err(e) => {
                    error!("Could not start Telegram. Error: {}", e);
                    return;
                }
            },
        ))),
        None => None,
    };

    if let Some(telegram_context) = telegram_context {
        tasks.push(Box::pin(async move {
            match telegram::run(telegram_context.clone()).await {
                Err(e) => Err(PlatformError::TelegramError(e)),
                Ok(_) => Ok(()),
            }
        }));
    }

    for result in futures::future::join_all(tasks).await {
        if let Err(e) = result {
            error!("Task exited with an error: {}", e);
        }
    }
}

fn save_dictionary(config: &Config, dict: &Dictionary) -> Result<(), dictionary::Error> {
    match dict.write_to_disk(Path::new(&config.dictionary_path)) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!(
                "Cannot write to dictionary file. Please ensure that the program \
                has the necessary permissions to write to the dictionary."
            );
            Err(e)
        }
    }
}
