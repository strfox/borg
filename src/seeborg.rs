use crate::config::Config;
use crate::dictionary::Dictionary;
use rand_core::SeedableRng;
use rand::rngs::SmallRng;

pub struct SeeBorg {
    pub config: Config,
    dictionary: Dictionary,
    rng: SmallRng,
}

impl SeeBorg {
    pub fn new(config: Config, dictionary: Dictionary) -> SeeBorg {
        SeeBorg {
            config: config,
            dictionary: dictionary,
            rng: SmallRng::from_entropy(),
        }
    }

    pub fn respond_to(&mut self, line: &str) -> Option<String> {
        self.dictionary.respond_to(line, &mut self.rng)
    }
}
