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
            return true;
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
            if !moves.is_empty() {
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
                if !moves.is_empty() {
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
            if !moves.is_empty() {
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
            if !moves.is_empty() {
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
                if !moves.is_empty() {
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
                let on_ray = !(get_bishop_rays(our_king) & their_bishops).is_empty();
                if on_ray && !(get_bishop_moves(our_king, blockers) & their_bishops).is_empty() {
                    continue;
                }
                let on_ray = !(get_rook_rays(our_king) & their_rooks).is_empty();
                if on_ray && !(get_rook_moves(our_king, blockers) & their_rooks).is_empty() {
                    continue;
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

    #[inline(always)]
    fn king_safe_on(&self, square: Square) -> bool {
        macro_rules! short_circuit {
            ($($attackers:expr),*) => {
                $(if !$attackers.is_empty() {
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
            let blockers = self.occupied() ^ our_king.bitboard();
            let pinned = self.pinned();
            let rights = self.castle_rights(color);
            let back_rank = Rank::First.relative_to(color);
            let mut handle_castling = |rook, king_dest, rook_dest| {
                let rook = Square::new(rook, back_rank);
                let blockers = blockers ^ rook.bitboard();
                let king_dest = Square::new(king_dest, back_rank);
                let rook_dest = Square::new(rook_dest, back_rank);
                let king_to_rook = get_between_rays(our_king, rook);
                let king_to_dest = get_between_rays(our_king, king_dest);
                let mut must_be_safe = king_to_dest | king_dest.bitboard();
                let must_be_empty = must_be_safe | king_to_rook | rook_dest.bitboard();
                let can_castle = !pinned.has(rook)
                    && (blockers & must_be_empty).is_empty()
                    && must_be_safe.all(|square| self.king_safe_on(square));
                if can_castle {
                    moves |= rook.bitboard();
                }
            };
            if let Some(rook) = rights.short {
                handle_castling(rook, File::G, File::F);
            }
            if let Some(rook) = rights.long {
                handle_castling(rook, File::C, File::D);
            }
        }
        if !moves.is_empty() {
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

    /// Generate all legal moves given a position in no particular order.
    /// All guarantees made by this function are only guaranteed if the board is valid.
    /// To retrieve the moves, a `listener` callback must be passed that receives compact [`PieceMoves`].
    /// This does *not* guarantee that each [`PieceMoves`] value has a unique `from` square.
    /// However, each [`PieceMoves`] value will have at least one move.
    /// The listener will be called a maximum of 18 times.
    /// The listener can abort the movegen early by returning `true`.
    /// In this case, this function also returns `true`.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let board = Board::default();
    /// let mut total_moves = 0;
    /// board.generate_moves(|moves| {
    ///     for _mv in moves {
    ///         total_moves += 1;
    ///     }
    ///     false
    /// });
    /// assert_eq!(total_moves, 20);
    /// ```
    pub fn generate_moves(&self, listener: impl FnMut(PieceMoves) -> bool) -> bool {
        self.try_generate_moves(listener).expect("Invalid board!")
    }

    pub fn try_generate_moves(&self, mut listener: impl FnMut(PieceMoves) -> bool) -> Result<bool, BoardError> {
        if self.try_king(self.side_to_move()).is_err() {
            return Err(BoardError::InvalidBoard);
        }
        Ok(match self.checkers().popcnt() {
            0 => self.add_all_legals::<_, false>(&mut listener),
            1 => self.add_all_legals::<_, true>(&mut listener),
            _ => self.add_king_legals::<_, true>(&mut listener)
        })
    }
}
