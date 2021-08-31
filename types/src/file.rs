use crate::*;

crate::helpers::simple_enum! {
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
