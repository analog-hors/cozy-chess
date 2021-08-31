use std::convert::TryInto;
use std::str::FromStr;

use crate::*;

crate::helpers::simple_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum Square {
        A1, B1, C1, D1, E1, F1, G1, H1,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A3, B3, C3, D3, E3, F3, G3, H3,
        A4, B4, C4, D4, E4, F4, G4, H4,
        A5, B5, C5, D5, E5, F5, G5, H5,
        A6, B6, C6, D6, E6, F6, G6, H6,
        A7, B7, C7, D7, E7, F7, G7, H7,
        A8, B8, C8, D8, E8, F8, G8, H8
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SquareParseError {
    InvalidSquare
}

impl FromStr for Square {
    type Err = SquareParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let file = chars.next()
            .and_then(|c| c.try_into().ok())
            .ok_or(SquareParseError::InvalidSquare)?;
        let rank = chars.next()
            .and_then(|c| c.try_into().ok())
            .ok_or(SquareParseError::InvalidSquare)?;
        let square = Square::new(file, rank);
        if chars.next().is_some() {
            Err(SquareParseError::InvalidSquare)
        } else {
            Ok(square)
        }
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl Square {
    #[inline(always)]
    pub const fn new(file: File, rank: Rank) -> Self {
        Self::index_const(((rank as usize) << 3) | file as usize)
    }

    #[inline(always)]
    pub const fn rank(self) -> Rank {
        Rank::index_const(self as usize >> 3)
    }

    #[inline(always)]
    pub const fn file(self) -> File {
        File::index_const(self as usize & 0b000_111)
    }

    ///Get a bitboard with this square set
    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(1 << self as u8)
    }
}
