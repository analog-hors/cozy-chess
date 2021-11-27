use crate::*;

crate::helpers::simple_enum! {
    /// A file on a chessboard
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum File {
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H
    }
}

crate::helpers::enum_char_conv! {
    File, FileParseError {
        A = 'a',
        B = 'b',
        C = 'c',
        D = 'd',
        E = 'e',
        F = 'f',
        G = 'g',
        H = 'h'
    }
}

impl File {
    /// Flip the file.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(File::A.flip(), File::H);
    /// ```
    #[inline(always)]
    pub const fn flip(self) -> Self {
        Self::index_const(Self::H as usize - self as usize)
    }

    /// Get a bitboard with all squares on this file set.
    /// # Examples
    /// ```
    /// # use cozy_chess_types::*;
    /// assert_eq!(File::B.bitboard(), bitboard! {
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    ///     . X . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(u64::from_ne_bytes([
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001
        ]) << self as usize)
    }
}
