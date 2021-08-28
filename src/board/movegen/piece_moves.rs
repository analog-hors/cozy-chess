use crate::*;

///A compact structure representing multiple moves for a piece on the board.
///Iterate it to unpack its moves.
#[derive(Debug, Clone, Copy)]
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

pub struct PieceMovesIter {
    moves: PieceMoves,
    promotion: u8
}

impl PieceMovesIter {
    pub fn next_dbg(&mut self) -> Option<Move> {
        while let Some(to) = self.moves.to.next_square() {
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
            return Some(Move {
                from: self.moves.from,
                to,
                promotion
            })
        }
        None
    }
}

impl Iterator for PieceMovesIter {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(to) = self.moves.to.next_square() {
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
            return Some(Move {
                from: self.moves.from,
                to,
                promotion
            })
        }
        None
    }
}
