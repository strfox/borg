extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate regex;
extern crate lazy_static;

mod config;
mod dictionary;

const CONFIG_PATH: &str = "config.yml";

fn main() {
    println!("SeeBorg5 by Michel Faria.");
    println!("Please wait while things are set up.");

    use std::path::Path;
    use config::{Config, ConfigError};

    let config = match Config::load(Path::new(CONFIG_PATH)) {
        Ok(c) => c,
        Err(e) => {
            match e {
                ConfigError::IOError(io_err) => {
                    println!("An I/O error happened and the program could not \
                    read the configuration file. Please make sure that the \
                    file exists and that the program has permissions to read \
                    it. Details: {:?}", io_err.to_string());
                    return;
                }
                ConfigError::YAMLError(yaml_err) => {
                    println!("A YAML parsing error occurred. This is most \
                    likely due to a malformed configuration file. Please check \
                    that your configuration is correct and try again. \
                    Details on the YAML parsing error: {:?}", yaml_err.to_string());
                    return;
                }
            }
        }
    };

    println!("Config loaded.");

    use dictionary::{Dictionary, DictionaryError};

    let mut dict = match Dictionary::load(Path::new(&config.dictionary_path)) {
        Ok(d) => d,
        Err(e) => match e {
            DictionaryError::IOError(io_err) => {
                println!("An I/O error happened while trying to read the dictionary \
                file, located at \"{:?}\". Please ensure that the file is present \
                at such location and make sure that this program has read and write \
                permissions. Details: {:?}", config.dictionary_path, io_err.to_string());
                return;
            },
            DictionaryError::JSONError(json_err) => {
                println!("A JSON parsing error occurred. This is most likely due to \
                a corrupted dictionary file. Please check the dictionary file for any \
                anomalies. Details on the JSON parsing error: {:?}", json_err.to_string());
                return;
            },
        }
    };

    println!("Dictionary loaded.");

    dict.sort_sentences()

    
}
