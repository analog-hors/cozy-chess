use crate::*;

crate::helpers::simple_enum! {
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
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(0b11111111 << (self as usize * 8))
    }

    pub const fn relative_to(self, color: Color) -> Self {
        if let Color::White = color {
            self
        } else {
            Self::index_const(Self::Eighth as usize - self as usize)
        }
    }
}
