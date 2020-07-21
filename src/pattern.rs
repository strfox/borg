use std::error;

use onig::Regex;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};

use crate::fmt;

#[derive(Debug, Clone)]
pub struct CompilationError {
    description: String,
}

impl From<onig::Error> for CompilationError {
    fn from(e: onig::Error) -> Self {
        CompilationError {
            description: e.description().to_string(),
        }
    }
}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Regex failed to compile: {}", self.description)
    }
}

impl error::Error for CompilationError {
    fn description(&self) -> &str {
        self.description.as_str()
    }
}

#[derive(Debug, Clone)]
pub struct NotCompiledError;

impl fmt::Display for NotCompiledError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "The regex is not compiled.")
    }
}

impl error::Error for NotCompiledError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pattern {
    #[serde(skip)]
    compiled: Option<Regex>,
    pub original: String,
}

impl Pattern {
    pub fn compile(&mut self) -> Result<&Regex, CompilationError> {
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

    pub fn get_regex(&self) -> Result<&Regex, NotCompiledError> {
        match self.compiled {
            Some(ref p) => Ok(p),
            None => Err(NotCompiledError),
        }
    }
}

pub(crate) fn matches_any<'a>(
    input: &str,
    patterns: &'a Vec<Pattern>,
) -> Option<&'a Pattern> {
    for p in patterns {
        match p.get_regex() {
            Ok(regex) => {
                if regex.is_match(input) {
                    return Some(p);
                }
            }
            Err(_e) => panic!("Pattern {:?} is not compiled", p),
        }
    }
    None
}
