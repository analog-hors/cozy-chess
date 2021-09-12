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
    ///Get a bitboard with all squares on this file set
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
