use onig::Regex;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use std::{error, fmt, fs, io, path::Path};

/////////////////////////////////////////////////////////////////////////////
// Configuration Error Type
/////////////////////////////////////////////////////////////////////////////

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

/////////////////////////////////////////////////////////////////////////////
// Configuration Types
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub dictionary_path: String,
    pub auto_save_period: i64,
    pub behavior: MainBehavior,
    pub telegram: Option<Platform>,
    pub discord: Option<Platform>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MainBehavior {
    pub speaking: bool,
    pub learning: bool,
    pub reply_rate: f32,
    pub reply_nick: f32,
    pub reply_magic: f32,
    pub nick_patterns: Vec<Pattern>,
    pub magic_patterns: Vec<Pattern>,
    pub blacklisted_patterns: Vec<Pattern>,
    pub ignored_users: Vec<Pattern>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OverrideBehavior {
    pub speaking: Option<bool>,
    pub learning: Option<bool>,
    pub reply_rate: Option<f32>,
    pub reply_nick: Option<f32>,
    pub reply_magic: Option<f32>,
    pub nick_patterns: Option<Vec<Pattern>>,
    pub magic_patterns: Option<Vec<Pattern>>,
    pub blacklisted_patterns: Option<Vec<Pattern>>,
    pub ignored_users: Option<Vec<Pattern>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatBehavior {
    pub chat_id: String,
    pub behavior: OverrideBehavior,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Platform {
    pub token: String,
    pub behavior: Option<OverrideBehavior>,
    pub chat_behaviors: Option<Vec<MainBehavior>>,
}

/////////////////////////////////////////////////////////////////////////////
// Config implementations
/////////////////////////////////////////////////////////////////////////////

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

/////////////////////////////////////////////////////////////////////////////
// Pattern
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum PatternError {
    CompilationError(onig::Error),
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PatternError::CompilationError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for PatternError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            PatternError::CompilationError(ref e) => Some(e),
        }
    }
}

impl From<onig::Error> for PatternError {
    fn from(err: onig::Error) -> PatternError {
        PatternError::CompilationError(err)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pattern {
    #[serde(skip)]
    compiled: Option<Regex>,
    pub original: String,
}

impl Pattern {
    pub fn regex(&mut self) -> Result<&Regex, PatternError> {
        match self.compiled {
            Some(ref p) => Ok(p),
            None => {
                self.compiled = Some(Regex::new(&self.original)?);
                // Since self.compiled was assigned a value in the previous
                // statement, it is safe to unwrap.
                Ok(self.compiled.as_ref().unwrap())
            }
        }
    }
}

fn matches_any(input: &str, patterns: &mut Vec<Pattern>) -> Result<bool, PatternError> {
    for p in patterns {
        match p.regex() {
            Ok(regex) => return Ok(regex.is_match(input)),
            Err(e) => return Err(e),
        }
    }
    Ok(false)
}

/////////////////////////////////////////////////////////////////////////////
// Behavior trait
/////////////////////////////////////////////////////////////////////////////

trait Behavior {
    /// compile_patterns should ompile all Pattern objects in the Behavior
    fn compile_patterns(&mut self) -> Result<(), PatternError>;
}

/////////////////////////////////////////////////////////////////////////////
// MainBehavior Implementations
/////////////////////////////////////////////////////////////////////////////

impl MainBehavior {
    pub fn triggers_reply_nick(&mut self, line: &str) -> Result<bool, PatternError> {
        matches_any(line, &mut self.blacklisted_patterns)
    }

    pub fn triggers_reply_magic(&mut self, line: &str) -> Result<bool, PatternError> {
        matches_any(line, &mut self.magic_patterns)
    }

    pub fn triggers_blacklist(&mut self, line: &str) -> Result<bool, PatternError> {
        matches_any(line, &mut self.blacklisted_patterns)
    }

    pub fn should_ignore(&mut self, user_id: &str) -> Result<bool, PatternError> {
        matches_any(user_id, &mut self.ignored_users)
    }
}

impl Behavior for MainBehavior {
    fn compile_patterns(&mut self) -> Result<(), PatternError> {
        for p in self
            .magic_patterns
            .iter_mut()
            .chain(self.blacklisted_patterns.iter_mut())
            .chain(self.nick_patterns.iter_mut())
        {
            p.regex()?;
        }
        Ok(())
    }
}

/////////////////////////////////////////////////////////////////////////////
// OverrideBehavior Implementations
/////////////////////////////////////////////////////////////////////////////

impl Behavior for OverrideBehavior {
    fn compile_patterns(&mut self) -> Result<(), PatternError> {
        if let Some(ref mut ps) = self.magic_patterns {
            for p in ps.iter_mut() {
                p.regex()?;
            }
        }
        if let Some(ref mut ps) = self.blacklisted_patterns {
            for p in ps.iter_mut() {
                p.regex()?;
            }
        }
        if let Some(ref mut ps) = self.nick_patterns {
            for p in ps.iter_mut() {
                p.regex()?;
            }
        }
        Ok(())
    }
}
