use crate::config::Config;
use crate::dictionary::Dictionary;
use std::cell::Cell;

pub struct SeeBorg {
    pub config: Config,
    pub dictionary: Cell<Dictionary>,
}

impl SeeBorg {
    pub fn new(config: Config, dictionary: Dictionary) -> SeeBorg {
        SeeBorg {
            config: config,
            dictionary: Cell::from(dictionary),
        }
    }
}
