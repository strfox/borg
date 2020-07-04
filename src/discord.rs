use crate::seeborg::SeeBorg;

struct Discord<'a> {
    seeborg: &'a SeeBorg,
}

impl Discord<'_> {
    fn new(seeborg: &SeeBorg) -> Discord {
        Discord { seeborg }
    }
}
