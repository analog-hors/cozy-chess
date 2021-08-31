use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastleRights {
    pub short: Option<File>,
    pub long: Option<File>
}

impl CastleRights {
    pub const EMPTY: CastleRights = CastleRights {
        short: None,
        long: None
    };
}
