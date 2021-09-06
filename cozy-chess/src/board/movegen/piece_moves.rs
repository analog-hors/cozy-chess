use crate::*;

///A compact structure representing multiple moves for a piece on the board.
///Iterate it to unpack its moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PieceMoves {
    pub piece: Piece,
    pub from: Square,
    pub to: BitBoard
}

impl IntoIterator for PieceMoves {
    type Item = Move;

    type IntoIter = PieceMovesIter;

    fn into_iter(self) -> Self::IntoIter {
        PieceMovesIter {
            moves: self,
            promotion: 0
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl PieceMoves {
    pub fn len(&self) -> usize {
        const PROMOTION_MASK: BitBoard = BitBoard(
            Rank::First.bitboard().0 | Rank::Eighth.bitboard().0
        );
        let moves = if self.piece == Piece::Pawn {
            (self.to & !PROMOTION_MASK).popcnt() +
            (self.to & PROMOTION_MASK).popcnt() * 4
        } else {
            self.to.popcnt()
        };
        moves as usize
    }

    pub fn empty(&self) -> bool {
        self.to == BitBoard::EMPTY
    }
}

pub struct PieceMovesIter {
    moves: PieceMoves,
    promotion: u8
}

impl Iterator for PieceMovesIter {
    type Item = Move;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(to) = self.moves.to.next_square() {
            let is_promotion = self.moves.piece == Piece::Pawn &&
                matches!(to.rank(), Rank::First | Rank::Eighth);
            let promotion = if is_promotion {
                let promotion = match self.promotion {
                    0 => Piece::Knight,
                    1 => Piece::Bishop,
                    2 => Piece::Rook,
                    3 => Piece::Queen,
                    _ => unreachable!()
                };
                if self.promotion < 3 {
                    self.promotion += 1;
                } else {
                    self.promotion = 0;
                    self.moves.to.next();
                }
                Some(promotion)
            } else {
                self.moves.to.next();
                None
            };
            Some(Move {
                from: self.moves.from,
                to,
                promotion
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for PieceMovesIter {
    fn len(&self) -> usize {
        self.moves.len() - self.promotion as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_handles_promotions() {
        let mv = PieceMoves {
            piece: Piece::Pawn,
            from: Square::A7,
            to: Square::A8.bitboard() | Square::B8.bitboard()
        };
        assert_eq!(mv.len(), 8);
        let mut iter = mv.into_iter();
        assert_eq!(iter.len(), 8);
        for len in (0..8).rev() {
            iter.next();
            assert_eq!(iter.len(), len);
        }
    }
}
