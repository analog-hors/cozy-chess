use crate::*;

/// A [bitboard](https://www.chessprogramming.org/Bitboards).
/// This represents some set of squares on a chessboard.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct BitBoard(pub u64);

macro_rules! impl_math_ops {
    ($($trait:ident,$fn:ident;)*) => {
        $(
            impl std::ops::$trait for BitBoard {
                type Output = Self;
    
                #[inline(always)]
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
                #[inline(always)]
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
    
                #[inline(always)]
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
                #[inline(always)]
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
            $(
                #[inline(always)]
                pub const fn $fn(self, other: Self) -> Self {
                    Self(self.0.$fn(other.0))
                }
            )*
        }
    };
}
impl_wrapping_ops!(wrapping_add, wrapping_mul, wrapping_sub, wrapping_div);

impl std::ops::Not for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitBoard {
    /// An empty [`BitBoard`].
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(BitBoard::EMPTY, bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// });
    /// ```
    pub const EMPTY: Self = Self(0);

    /// The edges on the board.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(BitBoard::EDGES, bitboard! {
    ///     X X X X X X X X
    ///     X . . . . . . X
    ///     X . . . . . . X
    ///     X . . . . . . X
    ///     X . . . . . . X
    ///     X . . . . . . X
    ///     X . . . . . . X
    ///     X X X X X X X X
    /// });
    /// ```
    pub const EDGES: BitBoard = bitboard! {
        X X X X X X X X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X . . . . . . X
        X X X X X X X X
    };

    /// Count the number of squares in the bitboard
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(BitBoard::EMPTY.popcnt(), 0);
    /// let bitboard = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// assert_eq!(bitboard.popcnt(), 12);
    /// ```
    #[inline(always)]
    pub const fn popcnt(self) -> u32 {
        self.0.count_ones()
    }

    /// Check if a [`Square`] is set.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// let bitboard = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// assert!(bitboard.has(Square::C3));
    /// assert!(!bitboard.has(Square::B2));
    /// ```
    #[inline(always)]
    pub const fn has(self, square: Square) -> bool {
        self.0 & square.bitboard().0 != BitBoard::EMPTY.0
    }

    /// Checks if the [`BitBoard`] is empty.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert!(BitBoard::EMPTY.is_empty());
    /// let bitboard = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// assert!(!bitboard.is_empty());
    /// ```
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == BitBoard::EMPTY.0
    }

    /// Grabs the least significant square, if it exists.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert!(BitBoard::EMPTY.next_square().is_none());
    /// let bitboard = bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . X X X . . .
    ///     . . X . X X . .
    ///     . . X X X X . .
    ///     . . X . X . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    /// };
    /// assert_eq!(bitboard.next_square(), Some(Square::C3));
    /// ```
    #[inline(always)]
    pub const fn next_square(self) -> Option<Square> {
        Square::try_index(self.0.trailing_zeros() as usize)
    }
}

impl Iterator for BitBoard {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let square = self.next_square();
        if let Some(square) = square {
            *self ^= square.bitboard();
        }
        square
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl ExactSizeIterator for BitBoard {
    #[inline(always)]
    fn len(&self) -> usize {
        self.popcnt() as usize
    }
}

/// [`BitBoard`] literal macro.
/// ```
/// # use cozy_chess_types::*;
/// let bb = bitboard! {
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     . . . X . . . .
///     X X X . X X X X
///     . . . X . . . .
///     . . . X . . . .
/// };
/// assert_eq!(bb, File::D.bitboard() ^ Rank::Third.bitboard());
/// ```
#[macro_export]
macro_rules! __bitboard {
    (
        $a8:tt $b8:tt $c8:tt $d8:tt $e8:tt $f8:tt $g8:tt $h8:tt
        $a7:tt $b7:tt $c7:tt $d7:tt $e7:tt $f7:tt $g7:tt $h7:tt
        $a6:tt $b6:tt $c6:tt $d6:tt $e6:tt $f6:tt $g6:tt $h6:tt
        $a5:tt $b5:tt $c5:tt $d5:tt $e5:tt $f5:tt $g5:tt $h5:tt
        $a4:tt $b4:tt $c4:tt $d4:tt $e4:tt $f4:tt $g4:tt $h4:tt
        $a3:tt $b3:tt $c3:tt $d3:tt $e3:tt $f3:tt $g3:tt $h3:tt
        $a2:tt $b2:tt $c2:tt $d2:tt $e2:tt $f2:tt $g2:tt $h2:tt
        $a1:tt $b1:tt $c1:tt $d1:tt $e1:tt $f1:tt $g1:tt $h1:tt
    ) => {
        $crate::__bitboard! { @raw
            $a1 $b1 $c1 $d1 $e1 $f1 $g1 $h1
            $a2 $b2 $c2 $d2 $e2 $f2 $g2 $h2
            $a3 $b3 $c3 $d3 $e3 $f3 $g3 $h3
            $a4 $b4 $c4 $d4 $e4 $f4 $g4 $h4
            $a5 $b5 $c5 $d5 $e5 $f5 $g5 $h5
            $a6 $b6 $c6 $d6 $e6 $f6 $g6 $h6
            $a7 $b7 $c7 $d7 $e7 $f7 $g7 $h7
            $a8 $b8 $c8 $d8 $e8 $f8 $g8 $h8
        }
    };
    (@raw $($occupied:tt)*) => {{
        let mut index = 0;
        let mut bitboard = $crate::BitBoard::EMPTY;
        $(
            if $crate::__bitboard!(@convert $occupied) {
                bitboard.0 |= 1 << index;
            }
            index += 1;
        )*
        let _ = index;
        bitboard
    }};
    (@convert X) => { true };
    (@convert .) => { false };
    (@convert $token:tt) => {
        compile_error!(
            concat!(
                "Expected only `X` or `.` tokens, found `",
                stringify!($token),
                "`"
            )
        )
    };
    ($($token:tt)*) => {
        compile_error!("Expected 64 squares")
    };
}
pub use __bitboard as bitboard;

impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &rank in Rank::ALL.iter().rev() {
            for &file in &File::ALL {
                let square = Square::new(file, rank).bitboard();
                if *self & square != Self::EMPTY {
                    write!(f, "X ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
