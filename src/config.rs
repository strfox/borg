use serde::{Deserialize, Serialize};
use std::fmt;
use std::error;
use std::io;
use std::fs;
use std::path::Path;



#[derive(Debug)]
pub enum ConfigError {
    IOError(io::Error),
    YAMLError(serde_yaml::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::IOError(ref e) => e.fmt(f),
            ConfigError::YAMLError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ConfigError::IOError(ref e) => Some(e),
            ConfigError::YAMLError(ref e) => Some(e),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::IOError(err)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> ConfigError {
        ConfigError::YAMLError(err)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Behavior {
    pub speaking: bool,
    pub learning: bool,
    pub reply_rate: f32,
    pub reply_nick: f32,
    pub reply_magic: f32,
    pub magic_patterns: Vec<String>,
    pub blacklisted_patterns: Vec<String>,
    pub ignored_users: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatBehavior {
    pub chat_id: String,
    pub behavior: Behavior,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Platform {
    pub token: String,
    pub behavior: Option<Behavior>,
    pub chat_behaviors: Option<Vec<Behavior>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub dictionary_path: String,
    pub auto_save_period: i64,
    pub behavior: Behavior,
    pub telegram: Option<Platform>,
    pub discord: Option<Platform>,
}

// NewConfig creates a new configuration file from a file at the given path.
// It will read the file from the disk and deserialize it into a Config struct.
// If an error occurs while reading the file, or while unmarshalling the the
// configuration data, it will return the error and leave the handling up to
// the caller.
impl Config {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let data = fs::read_to_string(&path)?;
        let config = serde_yaml::from_str(&data)?;
        Ok(config)
    }
}
