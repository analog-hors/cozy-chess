use crate::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct BitBoard(pub u64);

macro_rules! impl_math_ops {
    ($($trait:ident,$fn:ident;)*) => {
        $(
            impl std::ops::$trait for BitBoard {
                type Output = Self;
    
                fn $fn(self, other: Self) -> Self::Output {
                    Self(std::ops::$trait::$fn(self.0, other.0))
                }
            }
        )*
    };
}
impl_math_ops! {
    Add, add;
    Sub, sub;
    Mul, mul;
    Div, div;
    BitAnd, bitand;
    BitOr, bitor;
    BitXor, bitxor;
}

macro_rules! impl_math_assign_ops {
    ($($trait:ident,$fn:ident;)*) => {
        $(
            impl std::ops::$trait for BitBoard {
                fn $fn(&mut self, other: Self) {
                    std::ops::$trait::$fn(&mut self.0, other.0)
                }
            }
        )*
    };
}
impl_math_assign_ops! {
    AddAssign, add_assign;
    SubAssign, sub_assign;
    MulAssign, mul_assign;
    DivAssign, div_assign;
    BitAndAssign, bitand_assign;
    BitOrAssign, bitor_assign;
    BitXorAssign, bitxor_assign;
}

macro_rules! impl_shift_ops_for {
    ($type:ident; $($trait:ident,$fn:ident;)*) => {
        $(
            impl std::ops::$trait<$type> for BitBoard {
                type Output = Self;
    
                fn $fn(self, other: $type) -> Self::Output {
                    Self(std::ops::$trait::$fn(self.0, other))
                }
            }
        )*
    };
}
macro_rules! impl_shift_assign_ops_for {
    ($type:ident; $($trait:ident,$fn:ident;)*) => {
        $(
            impl std::ops::$trait<$type> for BitBoard {
                fn $fn(&mut self, other: $type) {
                    std::ops::$trait::$fn(&mut self.0, other)
                }
            }
        )*
    };
}
macro_rules! impl_shift_assign_ops {
    ($($type:ident),*) => {
        $(impl_shift_ops_for! {
            $type;
            Shl, shl;
            Shr, shr;
        }
    
        impl_shift_assign_ops_for! {
            $type;
            ShlAssign, shl_assign;
            ShrAssign, shr_assign;
        })*
    };
}
impl_shift_assign_ops!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);

macro_rules! impl_wrapping_ops {
    ($($fn:ident),*) => {
        impl BitBoard {
            $(pub const fn $fn(self, other: Self) -> Self {
                Self(self.0.$fn(other.0))
            })*
        }
    };
}
impl_wrapping_ops!(wrapping_add, wrapping_mul, wrapping_sub, wrapping_div);

impl std::ops::Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitBoard {
    pub const EMPTY: Self = Self(0);

    pub const EDGES: BitBoard = BitBoard(
        Rank::First.bitboard().0 |
        Rank::Eighth.bitboard().0 |
        File::A.bitboard().0 |
        File::H.bitboard().0
    );

    pub const fn popcnt(self) -> u32 {
        self.0.count_ones()
    }

    pub const fn has(self, square: Square) -> bool {
        self.0 & square.bitboard().0 != BitBoard::EMPTY.0
    }

    pub const fn empty(self) -> bool {
        self.0 == BitBoard::EMPTY.0
    }

    ///Grabs the least significant square, if it exists.
    pub const fn next_square(self) -> Option<Square> {
        Square::try_index(self.0.trailing_zeros() as usize)
    }
}

impl Iterator for BitBoard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let square = self.next_square();
        if let Some(square) = square {
            *self ^= square.bitboard();
        }
        square
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl ExactSizeIterator for BitBoard {
    fn len(&self) -> usize {
        self.popcnt() as usize
    }
}

impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &rank in Rank::ALL.iter().rev() {
            for &file in &File::ALL {
                let square = Square::new(file, rank).bitboard();
                if *self & square != Self::EMPTY {
                    write!(f, "X")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
