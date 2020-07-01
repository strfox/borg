use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

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

type Indices = HashMap<String, Vec<usize>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dictionary {
    sentences: Vec<String>,
    indices: Indices,
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

    fn reset_word_map(&mut self) {
        self.indices = HashMap::new();
    }

    pub fn needs_to_build_indices(&self) -> bool {
        self.sentences.len() > 0 && self.indices.len() == 0
    }

    pub fn rebuild_indices(&mut self) {
        self.reset_word_map();
        sort_sentences(&mut self.sentences);

        let mut indices: Indices = HashMap::new();
        use std::iter::repeat;
        self.sentences
            .iter()
            .enumerate()
            .map(|(i, sentence)| (i, sentence.to_lowercase()))
            .map(|(i, sentence)| (i, split_words(&sentence)))
            .flat_map(|(i, words)| repeat(i).zip(words.into_iter()))
            .for_each(|(i, word)| {
                let entry = indices.entry(word).or_insert_with(Vec::new);
                if !entry.contains(&i) {
                    entry.push(i);
                }
            });
        self.indices = indices
    }
}

fn split_sentences(s: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[.!?]+\s+").unwrap();
    }
    RE.split(s)
        .filter(|s| !s.is_empty())
        .map(|s| String::from(s))
        .collect()
}

fn split_words(s: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[,.!?:\s]+").unwrap();
    }
    RE.split(s)
        .filter(|s| !s.is_empty())
        .map(|s| String::from(s))
        .collect()
}

fn sort_sentences(sentences: &mut Vec<String>) {
    sentences.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_sentences() {
        assert_eq!(
            vec![
                "Hi",
                "This sentence is going to be split",
                "We.cant.split.things.that.look.like.urls",
                "That's a single sentence",
                "Lol",
                "A single sentence",
                "Look at this image: https://imgur.com/gallery/PXSNky0"
            ],
            split_sentences(
                "Hi. This sentence is going to be split. \
                We.cant.split.things.that.look.like.urls. That's a single sentence. \
                Lol! A single sentence!!!! Look at this image: https://imgur.com/gallery/PXSNky0"
            ),
        );
    }

    // This tests that the Dictionary::rebuild_indices function is building indices correctly.
    #[test]
    fn test_dictionary_rebuild_indices() {
        fn sample_sentences() -> Vec<String> {
            vec![
                "This is a test".to_string(),
                "This is is not a trick.".to_string(), // The double "is" is intentional
                "Hello world!".to_string(),
            ]
        }

        fn sample_sentences_sorted() -> Vec<String> {
            let mut sample_sentences = sample_sentences();
            sort_sentences(&mut sample_sentences);
            sample_sentences
        }

        // Set up test case
        let mut d = Dictionary::new_empty();
        d.sentences = sample_sentences();
        d.rebuild_indices();

        // Ensure that sentences are sorted.
        assert_eq!(sample_sentences_sorted(), d.sentences);

        // Ensure that the indices were correctly built
        let expected_indices: Indices = hashmap![
            "this".to_string() => vec![1, 2],
            "is".to_string() => vec![1, 2],
            "a".to_string() => vec![1, 2],
            "test".to_string() => vec![1],
            "not".to_string() => vec![2],
            "trick".to_string() => vec![2],
            "hello".to_string() => vec![0],
            "world".to_string() => vec![0]
        ];
        assert_eq!(expected_indices, d.indices);
    }

    #[test]
    fn test_split_words() {
        assert_eq!(
            vec!["Hello", "world", "This", "is", "a", "test", "I", "am", "a", "test"],
            split_words("...Hello world!!!!This is a test? I.am.a.test.")
        );
    }

    #[test]
    fn test_needs_to_build_indices() {
        let mut d = Dictionary::new_empty();

        // Indices should have to be rebuilt when the bot has sentences,
        // but no indices. On other conditions, it assumes that the indices
        // are correct and that they do not need to be rebuilt.

        // Has sentences but no indices
        d.sentences = vec!["Hello world".to_string()];
        assert!(d.needs_to_build_indices());

        // Has both sentences and indices
        d.sentences = vec!["Hello world".to_string()];
        d.indices = hashmap!["hello".to_string() => vec![0], "world".to_string() => vec![0]];
        assert!(!d.needs_to_build_indices());

        // Has no sentences and no indices
        d.sentences = vec![];
        d.indices = hashmap![];
        assert!(!d.needs_to_build_indices());
    }
}
