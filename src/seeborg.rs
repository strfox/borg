use crate::config::Behavior;
use crate::dictionary::Dictionary;
use crate::rand_core::RngCore;
use rand::rngs::SmallRng;
use rand_core::SeedableRng;

pub struct SeeBorg {
    behavior: Behavior,
    dictionary: Dictionary,
    rng: SmallRng,
}

/// This implementation is platform agnostic.
impl SeeBorg {
    pub fn new(behavior: Behavior, dictionary: Dictionary) -> SeeBorg {
        SeeBorg {
            behavior,
            dictionary,
            rng: SmallRng::from_entropy(),
        }
    }

    pub fn respond_to(&mut self, line: &str) -> Option<String> {
        self.dictionary.respond_to(line, &mut self.rng)
    }

    pub fn learn(&mut self, line: &str) {
        self.dictionary.learn(line);
    }

    pub fn reply_to<'a>(&'a mut self, user_id: &'a str) -> ReplyTo<'a> {
        ReplyTo {
            user_id,
            behavior: &self.behavior,
            rng: &mut self.rng,
        }
    }
}

pub struct ReplyTo<'a> {
    user_id: &'a str,
    behavior: &'a Behavior,
    rng: &'a mut SmallRng,
}

impl<'a> ReplyTo<'a> {
    pub fn override_behavior(&mut self, behavior: &'a Behavior) -> &mut ReplyTo<'a> {
        self.behavior = behavior;
        self
    }

    pub fn to_line(&self, line: &str) -> bool {
        if !self.behavior.is_speaking() || self.behavior.should_ignore(self.user_id) {
            false
        } else {
            
        }
    }
}

fn chance(chance: f32, rng: &mut SmallRng) -> bool {
    let p = rng.next_u32() % 100;
    p as f32 > chance || p == 100
}

impl Behavior {
    pub fn is_speaking(&self) -> bool {
        self.speaking
    }

    pub fn should_ignore(&self, user_id: &str) -> bool {
        self.ignored_users
            .iter()
            .any(|x| x == user_id)
    }
}