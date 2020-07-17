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
// PatternOwner trait
/////////////////////////////////////////////////////////////////////////////

/// Any struct that has Patterns in it can optionally implement this trait
/// to allow eager compilation of all patterns
trait PatternOwner {
    /// compile_patterns should compile all Pattern objects in the implementing
    /// struct.
    fn compile_patterns(&mut self) -> Result<(), PatternError>;
}

/////////////////////////////////////////////////////////////////////////////
// Config Struct
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub dictionary_path: String,
    pub auto_save_period: i64,
    pub behavior: MainBehavior,
    pub telegram: Option<TelegramPlatform>,
    pub discord: Option<DiscordPlatform>,
}

/////////////////////////////////////////////////////////////////////////////
// Config Implementations
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
// MainBehavior Struct
/////////////////////////////////////////////////////////////////////////////

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

/////////////////////////////////////////////////////////////////////////////
// MainBehavior Implementations
/////////////////////////////////////////////////////////////////////////////

impl PatternOwner for MainBehavior {
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
// OverrideBehavior Struct
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct BehaviorOverride {
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

/////////////////////////////////////////////////////////////////////////////
// OverrideBehavior Implementations
/////////////////////////////////////////////////////////////////////////////

impl PatternOwner for BehaviorOverride {
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

/////////////////////////////////////////////////////////////////////////////
// ChatBehavior Struct
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatBehaviorOverrides {
    pub chat_id: String,
    pub behavior: BehaviorOverride,
}

/////////////////////////////////////////////////////////////////////////////
// Telegram Struct
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramPlatform {
    pub token: String,
    pub behavior: Option<BehaviorOverride>,
    pub chat_behaviors: Option<Vec<ChatBehaviorOverrides>>,
    pub webhook_bind_address: String,
}

/////////////////////////////////////////////////////////////////////////////
// Discord Struct
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordPlatform {
    pub token: String,
    pub behavior: Option<BehaviorOverride>,
    pub chat_behaviors: Option<Vec<ChatBehaviorOverrides>>,
}

/////////////////////////////////////////////////////////////////////////////
// Pattern Error Type
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

/////////////////////////////////////////////////////////////////////////////
// Pattern Struct
/////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct Pattern {
    #[serde(skip)]
    compiled: Option<Regex>,
    pub original: String,
}

/////////////////////////////////////////////////////////////////////////////
// Pattern Implementations
/////////////////////////////////////////////////////////////////////////////

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
// BehaviorValues Struct
/////////////////////////////////////////////////////////////////////////////

pub struct BehaviorValueResolver<'a> {
    behavior: &'a MainBehavior,
    override_: Option<BehaviorOverrideValueResolver<'a>>,
}

/////////////////////////////////////////////////////////////////////////////
// BehaviorValues Implementations
/////////////////////////////////////////////////////////////////////////////

impl<'a> BehaviorValueResolver<'a> {
    pub fn new(
        behavior: &'a MainBehavior,
        override_: Option<BehaviorOverrideValueResolver<'a>>,
    ) -> BehaviorValueResolver<'a> {
        BehaviorValueResolver {
            behavior,
            override_,
        }
    }

    pub fn is_speaking(&self) -> bool {
        self.override_
            .as_ref()
            .and_then(|o| o.is_speaking())
            .unwrap_or(self.behavior.speaking)
    }

    pub fn is_learning(&self) -> bool {
        self.override_
            .as_ref()
            .and_then(|o| o.is_speaking())
            .unwrap_or(self.behavior.learning)
    }

    pub fn reply_rate(&self) -> f32 {
        self.override_
            .as_ref()
            .and_then(|o| o.reply_rate())
            .unwrap_or(self.behavior.reply_rate)
    }

    pub fn reply_magic(&self) -> f32 {
        self.override_
            .as_ref()
            .and_then(|o| o.reply_magic())
            .unwrap_or(self.behavior.reply_magic)
    }

    pub fn reply_nick(&self) -> f32 {
        self.override_
            .as_ref()
            .and_then(|o| o.reply_nick())
            .unwrap_or(self.behavior.reply_nick)
    }

    pub fn nick_patterns(&self) -> &Vec<Pattern> {
        self.override_
            .as_ref()
            .and_then(|o| o.nick_patterns())
            .unwrap_or(&self.behavior.nick_patterns)
    }

    pub fn magic_patterns(&self) -> &Vec<Pattern> {
        self.override_
            .as_ref()
            .and_then(|o| o.magic_patterns())
            .unwrap_or(&self.behavior.magic_patterns)
    }

    pub fn blacklisted_patterns(&self) -> &Vec<Pattern> {
        self.override_
            .as_ref()
            .and_then(|o| o.blacklisted_patterns())
            .unwrap_or(&self.behavior.blacklisted_patterns)
    }

    pub fn ignored_users(&self) -> &Vec<Pattern> {
        self.override_
            .as_ref()
            .and_then(|o| o.ignored_users())
            .unwrap_or(&self.behavior.ignored_users)
    }
}

/////////////////////////////////////////////////////////////////////////////
// OverrideResolver Struct
/////////////////////////////////////////////////////////////////////////////

pub struct BehaviorOverrideValueResolver<'a> {
    behavior: &'a BehaviorOverride,
    override_: Option<Box<BehaviorOverrideValueResolver<'a>>>,
}

/////////////////////////////////////////////////////////////////////////////
// OverrideResolver Implementations
/////////////////////////////////////////////////////////////////////////////

impl<'a> BehaviorOverrideValueResolver<'a> {
    pub fn new(
        behavior: &'a BehaviorOverride,
        override_: Option<Box<BehaviorOverrideValueResolver<'a>>>,
    ) -> BehaviorOverrideValueResolver<'a> {
        BehaviorOverrideValueResolver {
            behavior,
            override_,
        }
    }

    pub fn is_speaking(&self) -> Option<bool> {
        self.override_
            .as_ref()
            .map(|o| o.is_speaking())
            .unwrap_or(self.behavior.speaking)
    }

    pub fn is_learning(&self) -> Option<bool> {
        self.override_
            .as_ref()
            .map(|o| o.is_speaking())
            .unwrap_or(self.behavior.learning)
    }

    pub fn reply_rate(&self) -> Option<f32> {
        self.override_
            .as_ref()
            .map(|o| o.reply_rate())
            .unwrap_or(self.behavior.reply_rate)
    }

    pub fn reply_magic(&self) -> Option<f32> {
        self.override_
            .as_ref()
            .map(|o| o.reply_magic())
            .unwrap_or(self.behavior.reply_magic)
    }

    pub fn reply_nick(&self) -> Option<f32> {
        self.override_
            .as_ref()
            .map(|o| o.reply_nick())
            .unwrap_or(self.behavior.reply_nick)
    }

    pub fn nick_patterns(&self) -> Option<&Vec<Pattern>> {
        self.override_
            .as_ref()
            .map(|o| o.nick_patterns())
            .unwrap_or(self.behavior.nick_patterns.as_ref())
    }

    pub fn magic_patterns(&self) -> Option<&Vec<Pattern>> {
        self.override_
            .as_ref()
            .map(|o| o.magic_patterns())
            .unwrap_or(self.behavior.magic_patterns.as_ref())
    }

    pub fn blacklisted_patterns(&self) -> Option<&Vec<Pattern>> {
        self.override_
            .as_ref()
            .map(|o| o.blacklisted_patterns())
            .unwrap_or(self.behavior.blacklisted_patterns.as_ref())
    }

    pub fn ignored_users(&self) -> Option<&Vec<Pattern>> {
        self.override_
            .as_ref()
            .map(|o| o.ignored_users())
            .unwrap_or(self.behavior.ignored_users.as_ref())
    }
}
