use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct SquareDelta(pub i8, pub i8);

impl SquareDelta {
    #[inline(always)]
    pub const fn add(self, square: Square) -> Option<Square> {
        macro_rules! const_try {
            ($expr:expr) => {{
                //If we write it as an expression, clippy complains we can
                //use ? even though we can't because it's a const context.
                //So we have to convert it to this to stick on
                //#[allow(clippy::question_mark)], because otherwise the
                //compiler complains. This causes the clippy warning to go
                //away anyway. Bleh.
                let ret;
                #[allow(clippy::question_mark)]
                if let Some(value) = $expr {
                    ret = value;
                } else {
                    return None;
                }
                ret
            }};
        }
        Some(Square::new(
            const_try!(File::try_index((square.file() as i8 + self.0) as usize)),
            const_try!(Rank::try_index((square.rank() as i8 + self.1) as usize))
        ))
    }
}
