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
mod util;
mod config;
mod dictionary;
mod discord;
mod seeborg;
mod telegram;

use config::{Config, ConfigError};
use dictionary::{Dictionary, DictionaryError};
use futures::lock::Mutex;
use futures::Future;
use seeborg::SeeBorg;
use std::error;
use std::fmt;
use std::path::Path;
use std::sync::Arc;
use telegram::Telegram;
use std::pin::Pin;

const CONFIG_PATH: &str = "config.yml";

#[derive(Debug)]
pub enum PlatformError {
    TelegramError(telegram_bot::Error),
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

impl From<telegram_bot::Error> for PlatformError {
    fn from(err: telegram_bot::Error) -> PlatformError {
        PlatformError::TelegramError(err)
    }
}

#[tokio::main]
async fn main() {
    println!("SeeBorg5 by Michel Faria.");
    println!("Please wait while things are set up.");

    let config = match Config::load(Path::new(CONFIG_PATH)) {
        Ok(c) => c,
        Err(e) => match e {
            ConfigError::IOError(io_err) => {
                println!(
                    "An I/O error happened and the program could not \
                    read the configuration file. Please make sure that the \
                    file exists and that the program has permissions to read \
                    it. Details: {:?}",
                    io_err.to_string()
                );
                return;
            }
            ConfigError::YAMLError(yaml_err) => {
                println!(
                    "A YAML parsing error occurred. This is most \
                    likely due to a malformed configuration file. Please check \
                    that your configuration is correct and try again. \
                    Details on the YAML parsing error: {:?}",
                    yaml_err.to_string()
                );
                return;
            }
        },
    };

    println!("{:?} loaded.", CONFIG_PATH);

    let mut dict = match Dictionary::load(Path::new(&config.dictionary_path)) {
        Ok(d) => d,
        Err(e) => match e {
            DictionaryError::IOError(io_err) => {
                println!(
                    "An I/O error happened while trying to read the dictionary \
                file, located at \"{:?}\". Please ensure that the file is present \
                at such location and make sure that this program has read and write \
                permissions. Details: {:?}",
                    config.dictionary_path,
                    io_err.to_string()
                );
                return;
            }
            DictionaryError::JSONError(json_err) => {
                println!(
                    "A JSON parsing error occurred. This is most likely due to \
                a corrupted dictionary file. Please check the dictionary file for any \
                anomalies. Details on the JSON parsing error: {:?}",
                    json_err.to_string()
                );
                return;
            }
        },
    };

    println!("{:?} loaded.", &config.dictionary_path);

    if dict.needs_to_build_indices() {
        println!("Indices need to be built. Building indices.");
        dict.rebuild_indices();
        println!("Indices built.");
    }

    let bot = Arc::new(Mutex::new(SeeBorg::new(config, dict)));
    let mut tasks: Vec<Pin<Box<dyn Future<Output = Result<(), PlatformError>>>>> = vec![];

    let telegram = if bot.lock().await.config.telegram.is_some() {
        Some(Arc::new(Mutex::new(Telegram::new(bot).await)))
    } else {
        None
    };

    if let Some(shared_t) = telegram {
        tasks.push(Box::pin(async move {
            let mut telegram = shared_t.lock().await;
            telegram.poll().await
        }));
    }

    futures::future::join_all(tasks).await;
}

fn save_dictionary(config: Config, dict: Dictionary) -> Result<(), DictionaryError> {
    match dict.write_to_disk(Path::new(&config.dictionary_path)) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!(
                "Error: Cannot write to dictionary file. Please ensure that the program \
                has the necessary permissions to write to the dictionary."
            );
            Err(e)
        }
    }
}
