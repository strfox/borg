use crate::{config::BehaviorOverrideValueResolver, dictionary::Dictionary, rand_core::RngCore};
use rand::rngs::SmallRng;
use rand_core::SeedableRng;

/////////////////////////////////////////////////////////////////////////////
// Borg Type
/////////////////////////////////////////////////////////////////////////////

pub struct Borg {
    dictionary: Dictionary,
    rng: SmallRng,
}

/////////////////////////////////////////////////////////////////////////////
// Borg Implementations
/////////////////////////////////////////////////////////////////////////////

/// This implementation is platform agnostic.
impl Borg {
    pub fn new(dictionary: Dictionary) -> Borg {
        Borg {
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

    pub fn should_learn(
        &mut self,
        user_id: &str,
        behavior: Option<&BehaviorOverrideValueResolver>,
    ) -> bool {
        true // TODO
    }

    pub fn should_reply_to(
        &mut self,
        user_id: &str,
        behavior: Option<&BehaviorOverrideValueResolver>,
    ) -> bool {
        true // TODO
    }
}

fn chance(chance: f32, rng: &mut SmallRng) -> bool {
    let p = rng.next_u32() % 100;
    p as f32 > chance || p == 100
}
