use crate::*;

use super::*;

mod piece_moves;

pub use piece_moves::*;

#[cfg(test)]
mod tests;

mod slider {
    use super::*;

    pub trait SlidingPiece {
        const PIECE: Piece;

        fn pseudo_legals(square: Square, blockers: BitBoard) -> BitBoard;
    }

    macro_rules! impl_sliding_piece {
        ($square:ident,$color:ident,$blockers:ident; $($type:ident => $impl:expr),*) => {
            $(pub struct $type;

            impl SlidingPiece for $type {
                const PIECE: Piece = Piece::$type;

                fn pseudo_legals($square: Square, $blockers: BitBoard) -> BitBoard {
                    $impl
                }
            })*
        };
    }

    impl_sliding_piece! {
        square, color, blockers;
        Bishop => get_bishop_moves(square, blockers),
        Rook => get_rook_moves(square, blockers),
        Queen => get_bishop_moves(square, blockers) | get_rook_moves(square, blockers)
    }
}

macro_rules! abort_if {
    ($($expr:expr),*) => {
        $(if $expr {
            return false;
        })*
    }
}

impl Board {
    //Squares we can land on. When we're in check, we have to block
    //or capture the checker. In any case, we can't land on our own
    //pieces. Assumed to only be called if there is only one checker.
    fn target_squares<const IN_CHECK: bool>(&self) -> BitBoard {
        let color = self.side_to_move();
        let targets = if IN_CHECK {
            let checker = self.checkers().next_square().unwrap();
            let our_king = self.king(color);
            get_between_rays(checker, our_king) | checker.bitboard()
        } else {
            !BitBoard::EMPTY
        };
        targets & !self.colors(color)
    }

    fn add_slider_legals<
        P: slider::SlidingPiece, F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool
    >(&self, listener: &mut F) -> bool {
        let color = self.side_to_move();
        let our_king = self.king(color);
        let pieces = self.pieces(P::PIECE) & self.colors(color);
        let pinned = self.pinned();
        let blockers = self.occupied();
        let target_squares = self.target_squares::<IN_CHECK>();

        for piece in pieces & !pinned {
            let moves = P::pseudo_legals(piece, blockers) & target_squares;
            if !moves.empty() {
                abort_if!(listener(PieceMoves {
                    piece: P::PIECE,
                    from: piece,
                    to: moves
                }));
            }
        }

        if !IN_CHECK {
            for piece in pieces & pinned {
                //If we're not in check, we can still slide along the pinned ray.
                let target_squares = target_squares & get_line_rays(our_king, piece);
                let moves = P::pseudo_legals(piece, blockers) & target_squares;
                if !moves.empty() {
                    abort_if!(listener(PieceMoves {
                        piece: P::PIECE,
                        from: piece,
                        to: moves
                    }));
                }
            }
        }
        false
    }

    fn add_knight_legals<F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool>(&self, listener: &mut F) -> bool {
        const PIECE: Piece = Piece::Knight;

        let color = self.side_to_move();
        let pieces = self.pieces(PIECE) & self.colors(color);
        let pinned = self.pinned();
        let target_squares = self.target_squares::<IN_CHECK>();

        for piece in pieces & !pinned {
            let moves = get_knight_moves(piece) & target_squares;
            if !moves.empty() {
                abort_if!(listener(PieceMoves {
                    piece: PIECE,
                    from: piece,
                    to: moves
                }));
            }
        }
        false
    }

    fn add_pawn_legals<F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool>(&self, listener: &mut F) -> bool {
        const PIECE: Piece = Piece::Pawn;

        let color = self.side_to_move();
        let our_king = self.king(color);
        let pieces = self.pieces(PIECE) & self.colors(color);
        let their_pieces = self.colors(!color);
        let pinned = self.pinned();
        let blockers = self.occupied();
        let target_squares = self.target_squares::<IN_CHECK>();

        for piece in pieces & !pinned {
            let moves = (
                get_pawn_quiets(piece, color, blockers) |
                (get_pawn_attacks(piece, color) & their_pieces)
            ) & target_squares;
            if !moves.empty() {
                abort_if!(listener(PieceMoves {
                    piece: PIECE,
                    from: piece,
                    to: moves
                }));
            }
        }

        if !IN_CHECK {
            for piece in pieces & pinned {
                //If we're not in check, we can still slide along the pinned ray.
                let target_squares = target_squares & get_line_rays(our_king, piece);
                let moves = (
                    get_pawn_quiets(piece, color, blockers) |
                    (get_pawn_attacks(piece, color) & their_pieces)
                ) & target_squares;
                if !moves.empty() {
                    abort_if!(listener(PieceMoves {
                        piece: PIECE,
                        from: piece,
                        to: moves
                    }));
                }
            }
        }

        if let Some(en_passant) = self.en_passant() {
            let their_bishops = their_pieces & (
                self.pieces(Piece::Bishop) |
                self.pieces(Piece::Queen)
            );
            let their_rooks = their_pieces & (
                self.pieces(Piece::Rook) |
                self.pieces(Piece::Queen)
            );

            let dest = Square::new(en_passant, Rank::Third.relative_to(!color));
            let victim = Square::new(en_passant, Rank::Fourth.relative_to(!color));
            for piece in get_pawn_attacks(dest, !color) & pieces {
                //Simulate the capture and update the pieces accordingly.
                let blockers = blockers
                    ^ victim.bitboard()
                    ^ piece.bitboard()
                    | dest.bitboard();
                //First test a basic ray to prevent an expensive magic lookup
                if !(get_bishop_rays(our_king) & their_bishops).empty() {
                    if !(get_bishop_moves(our_king, blockers) & their_bishops).empty() {
                        continue;
                    }
                }
                if !(get_rook_rays(our_king) & their_rooks).empty() {
                    if !(get_rook_moves(our_king, blockers) & their_rooks).empty() {
                        continue;
                    }
                }
                abort_if!(listener(PieceMoves {
                    piece: PIECE,
                    from: piece,
                    to: dest.bitboard()
                }));
            }
        }
        false
    }

    fn king_safe_on(&self, square: Square) -> bool {
        macro_rules! short_circuit {
            ($($attackers:expr),*) => {
                $(if !$attackers.empty() {
                    return false;
                })*
                true
            }
        }

        let color = self.side_to_move();
        let their_pieces = self.colors(!color);
        let blockers = self.occupied()
            ^ (self.pieces(Piece::King) & self.colors(color))
            | square.bitboard();
        short_circuit! {
            get_bishop_moves(square, blockers) & their_pieces & (
                self.pieces(Piece::Bishop) | self.pieces(Piece::Queen)
            ),
            get_rook_moves(square, blockers) & their_pieces & (
                self.pieces(Piece::Rook) | self.pieces(Piece::Queen)
            ),
            get_knight_moves(square) & their_pieces & self.pieces(Piece::Knight),
            get_king_moves(square) & their_pieces & self.pieces(Piece::King),
            get_pawn_attacks(square, color) & their_pieces & self.pieces(Piece::Pawn)
        }
    }

    fn add_king_legals<F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool>(&self, listener: &mut F) -> bool {
        const PIECE: Piece = Piece::King;

        let color = self.side_to_move();
        let our_pieces = self.colors(color);
        let our_king = self.king(color);
        let mut moves = BitBoard::EMPTY;
        for to in get_king_moves(our_king) & !our_pieces {
            if self.king_safe_on(to) {
                moves |= to.bitboard();
            }
        }
        if !IN_CHECK {
            let blockers = self.occupied();
            let pinned = self.pinned();
            let rights = self.castle_rights(color);
            let back_rank = Rank::First.relative_to(color);
            if let Some(rook) = rights.short {
                let rook = Square::new(rook, back_rank);
                if !pinned.has(rook) && (blockers & get_between_rays(our_king, rook)).empty() {
                    let mut path = get_between_rays(our_king, Square::new(File::H, back_rank));
                    if path.all(|square| self.king_safe_on(square)) {
                        moves |= rook.bitboard();
                    }
                }
            }
            if let Some(rook) = rights.long {
                let rook = Square::new(rook, back_rank);
                if !pinned.has(rook) && (blockers & get_between_rays(our_king, rook)).empty() {
                    let mut path = get_between_rays(our_king, Square::new(File::B, back_rank));
                    if path.all(|square| self.king_safe_on(square)) {
                        moves |= rook.bitboard();
                    }
                }
            }
        }
        if !moves.empty() {
            abort_if!(listener(PieceMoves {
                piece: PIECE,
                from: our_king,
                to: moves
            }));
        }
        false
    }

    fn add_all_legals<F: FnMut(PieceMoves) -> bool, const IN_CHECK: bool>(&self, listener: &mut F) -> bool {
        abort_if! {
            self.add_pawn_legals::<_, IN_CHECK>(listener),
            self.add_knight_legals::<_, IN_CHECK>(listener),
            self.add_slider_legals::<slider::Bishop, _, IN_CHECK>(listener),
            self.add_slider_legals::<slider::Rook, _, IN_CHECK>(listener),
            self.add_slider_legals::<slider::Queen, _, IN_CHECK>(listener),
            self.add_king_legals::<_, IN_CHECK>(listener)
        }
        false
    }

    ///Generate all legal moves given a position in no particular order.
    ///All guarantees made by this function are only guaranteed if the board is valid.
    ///To retrieve the moves, a `listener` callback must be passed that receives compact [`PieceMoves`].
    ///This does *not* guarantee that each [`PieceMoves`] value has a unique `from` square.
    ///However, each [`PieceMoves`] value will have at least one move.
    ///The listener can abort the movegen early by returning `true`.
    ///In this case, this function also returns `true`.
    pub fn generate_moves(&self, listener: &mut impl FnMut(PieceMoves) -> bool) -> bool {
        match self.checkers().popcnt() {
            0 => self.add_all_legals::<_, false>(listener),
            1 => self.add_all_legals::<_, true>(listener),
            _ => self.add_king_legals::<_, true>(listener)
        }
    }
}
