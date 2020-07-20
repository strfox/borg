use crate::borg::Borg;

struct Discord<'a> {
    borg: &'a Borg,
}

impl Discord<'_> {
    fn new(borg: &Borg) -> Discord {
        Discord { borg }
    }
}
