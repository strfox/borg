use crate::{
    config::{BehaviorOverrideValueResolver, BehaviorValueResolver, MainBehavior},
    dictionary::Dictionary,
    rand_core::RngCore,
};
use rand::rngs::SmallRng;
use rand_core::SeedableRng;

/////////////////////////////////////////////////////////////////////////////
// SeeBorg Type
/////////////////////////////////////////////////////////////////////////////

pub struct SeeBorg {
    dictionary: Dictionary,
    rng: SmallRng,
}

/////////////////////////////////////////////////////////////////////////////
// SeeBorg Implementations
/////////////////////////////////////////////////////////////////////////////

/// This implementation is platform agnostic.
impl SeeBorg {
    pub fn new(dictionary: Dictionary) -> SeeBorg {
        SeeBorg {
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

    pub fn should_reply_to(
        &mut self,
        user_id: &str,
        behavior: Option<&BehaviorOverrideValueResolver>,
    ) -> bool {
        todo!()
    }
}

fn chance(chance: f32, rng: &mut SmallRng) -> bool {
    let p = rng.next_u32() % 100;
    p as f32 > chance || p == 100
}
