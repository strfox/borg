use crate::config::{BehaviorValueResolver, MainBehavior};
use crate::{
    config::BehaviorOverrideValueResolver, dictionary::Dictionary, pattern, rand_core::RngCore,
};
use rand::rngs::SmallRng;
use rand_core::SeedableRng;
use crate::pattern::NotCompiledError;

/////////////////////////////////////////////////////////////////////////////
// Borg Type
/////////////////////////////////////////////////////////////////////////////

pub struct Borg {
    dictionary: Dictionary,
    behavior: MainBehavior,
    rng: SmallRng,
}

/////////////////////////////////////////////////////////////////////////////
// Borg Implementations
/////////////////////////////////////////////////////////////////////////////

/// This implementation is platform agnostic.
impl Borg {
    pub fn new(dictionary: Dictionary, behavior: MainBehavior) -> Borg {
        Borg {
            dictionary,
            behavior,
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
        behavior: &Option<BehaviorOverrideValueResolver>,
    ) -> bool {
        true // TODO
    }

    pub fn should_reply_to(
        &mut self,
        user_id: &str,
        input: &str,
        behavior: &Option<BehaviorOverrideValueResolver>,
    ) -> Result<bool, NotCompiledError> {
        let b = BehaviorValueResolver::new(&self.behavior, behavior);

        debug!("Using {:?} for resolving behavior values.", b);

        match pattern::matches_any(user_id, b.ignored_users()) {
            Ok(result) => {
                if let Some(matched) = result {
                    debug!(
                        "User is ignored, user ID {:?} matched pattern {:?}",
                        user_id, matched
                    );
                    return Ok(true);
                }
            }
            Err(e) => return Err(e)
        }

        if !b.is_speaking() {
            debug!("Speaking is off");
            return Ok(false);
        }

        match pattern::matches_any(input, b.nick_patterns()) {
            Ok(result) => {
                if let Some(matched) = result {
                    debug!("Input \"{:?}\" matched nick pattern {:?}", input, matched);
                    let reply_nick = b.reply_nick();
                    debug!("Reply to nickname chance: {:?}", reply_nick);
                    if chance(reply_nick, &mut self.rng) {
                        debug!("Reply nick decided to reply");
                        return Ok(true);
                    } else {
                        debug!("Reply nick decided not to reply")
                    }
                }
            }
            Err(e) => return Err(e)
        }

        match pattern::matches_any(input, b.magic_patterns()) {
            Ok(result) => {
                if let Some(matched) = result {
                    debug!("Input \"{:?}\" matched magic pattern {:?}", input, matched);
                    let reply_magic = b.reply_magic();
                    debug!("Reply to magic patterns chance: {:?}", reply_magic);
                    if chance(reply_magic, &mut self.rng) {
                        debug!("Reply magic decided to reply");
                        return Ok(true);
                    } else {
                        debug!("Reply magic decided not to reply");
                    }
                }
            }
            Err(e) => return Err(e)
        }

        let reply_rate = b.reply_rate();
        debug!("Reply rate: {:?}", reply_rate);
        return Ok(if chance(reply_rate, &mut self.rng) {
            debug!("Decided to reply to reply rate");
            true
        } else {
            debug!("Decided not to reply to reply rate");
            false
        })
    }
}

fn chance(chance: f32, rng: &mut SmallRng) -> bool {
    let p = rng.next_u32() % 100;
    p as f32 > chance || p == 100
}
