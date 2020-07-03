#[macro_use]
extern crate lazy_static;
extern crate onig;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate futures;
extern crate tokio;

#[macro_use]
mod util;
mod config;
mod dictionary;
mod seeborg;
mod telegram;
mod discord;

use config::{Config, ConfigError};
use dictionary::{Dictionary, DictionaryError};
use std::path::Path;
use seeborg::SeeBorg;
use telegram::Telegram;

const CONFIG_PATH: &str = "config.yml";

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

    let seeborg = SeeBorg::new(config, dict);

    if seeborg.config.telegram.is_some() {
        println!("Enabling Telegram.");
        let telegram = Telegram::new(&seeborg);
        match telegram.poll().await {
            Ok(_) => (),
            Err(e) => println!("Error occurred in Telegram: ${:?}", e)
        };
    } 
    
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