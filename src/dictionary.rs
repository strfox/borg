use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use regex::Regex;

#[derive(Debug)]
pub enum DictionaryError {
    IOError(io::Error),
    JSONError(serde_json::Error),
}

impl fmt::Display for DictionaryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DictionaryError::IOError(ref e) => e.fmt(f),
            DictionaryError::JSONError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for DictionaryError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            DictionaryError::IOError(ref e) => Some(e),
            DictionaryError::JSONError(ref e) => Some(e),
        }
    }
}

impl From<io::Error> for DictionaryError {
    fn from(err: io::Error) -> DictionaryError {
        DictionaryError::IOError(err)
    }
}

impl From<serde_json::Error> for DictionaryError {
    fn from(err: serde_json::Error) -> DictionaryError {
        DictionaryError::JSONError(err)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dictionary {
    sentences: Vec<String>,
    indices: HashMap<String, Vec<i64>>,
}

impl Dictionary {
    // load loads a dictionary from the specified path.
    // If there is no file at the specified path, it will create a blank
    // dictionary at that location.
    pub fn load(path: &Path) -> Result<Self, DictionaryError> {
        if !path.is_file() {
            let d = Dictionary::new_empty();
            d.write_to_disk(&path)?;
            Ok(d)
        } else {
            let data = fs::read_to_string(path)?;
            let dict: Dictionary = serde_json::from_str(&data)?;
            Ok(dict)
        }
    }

    pub fn write_to_disk(&self, path: &Path) -> Result<(), DictionaryError> {
        let json = serde_json::to_string(&self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn new_empty() -> Dictionary {
        Dictionary {
            sentences: vec![],
            indices: HashMap::new(),
        }
    }

    pub fn sort_sentences(&mut self) {
        self.sentences
            .sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    }

    fn reset_word_map(&mut self) {
        self.indices = HashMap::new();
    }

    pub fn build_indices(&mut self) {
        self.reset_word_map();
        self.sort_sentences();

        lazy_static!
        let split_sentences = Regex::new(r"(?<=[.!?])(\s+)").unwrap();

        self.sentences.iter()
            .map(|s| s.split())
    }
}
