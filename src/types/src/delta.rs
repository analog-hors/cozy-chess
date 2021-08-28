use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct SquareDelta(pub i8, pub i8);

impl SquareDelta {
    pub const fn add(self, square: Square) -> Option<Square> {
        macro_rules! const_try {
            ($expr:expr) => {
                if let Some(value) = $expr {
                    value
                } else {
                    return None;
                }
            };
        }
        Some(Square::new(
            const_try!(File::try_index((square.file() as i8 + self.0) as usize)),
            const_try!(Rank::try_index((square.rank() as i8 + self.1) as usize))
        ))
    }
}
