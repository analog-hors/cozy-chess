use std::str::FromStr;

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Piece>
}

#[derive(Debug, Clone, Copy)]
pub enum MoveParseError {
    InvalidMove
}

impl FromStr for Move {
    type Err = MoveParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse(s: &str) -> Option<Move> {
            Some(Move {
                from: s.get(0..2)?.parse().ok()?,
                to: s.get(2..4)?.parse().ok()?,
                promotion: if let Some(promotion) = s.get(4..5) {
                    let promotion = promotion.parse().ok()?;
                    if matches!(promotion, Piece::King | Piece::Pawn) {
                        None
                    } else {
                        Some(promotion)
                    }
                } else {
                    None
                }
            })
        }
        parse(s).ok_or(MoveParseError::InvalidMove)
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(promotion) = self.promotion {
            write!(f, "{}", promotion)?;
        }
        Ok(())
    }
}
