use crate::*;

crate::helpers::simple_enum! {
    /// A rank on a chessboard.
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Rank {
        First,
        Second,
        Third,
        Fourth,
        Fifth,
        Sixth,
        Seventh,
        Eighth
    }
}

crate::helpers::enum_char_conv! {
    Rank, RankParseError {
        First = '1',
        Second = '2',
        Third = '3',
        Fourth = '4',
        Fifth = '5',
        Sixth = '6',
        Seventh = '7',
        Eighth = '8'
    }
}

impl Rank {
    /// Flip the rank.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Rank::First.flip(), Rank::Eighth);
    /// ```
    #[inline(always)]
    pub const fn flip(self) -> Self {
        Self::index_const(Self::Eighth as usize - self as usize)
    }

    /// Get a bitboard with all squares on this rank set.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Rank::Second.bitboard(), bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     X X X X X X X X
    ///     . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(0b11111111 << (self as usize * 8))
    }

    /// Get a rank relative to some color. This effectively
    /// flips the rank if viewing from black's perspective.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(Rank::First.relative_to(Color::White), Rank::First);
    /// assert_eq!(Rank::First.relative_to(Color::Black), Rank::Eighth);
    /// ```
    #[inline(always)]
    pub const fn relative_to(self, color: Color) -> Self {
        if let Color::White = color {
            self
        } else {
            self.flip()
        }
    }
}
