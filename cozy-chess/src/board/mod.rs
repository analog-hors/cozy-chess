use crate::*;

mod movegen;
mod parse;
mod zobrist;

use zobrist::*;
pub use movegen::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GameStatus {
    Won,
    Drawn,
    Ongoing
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    inner: ZobristBoard,
    pinned: BitBoard,
    checkers: BitBoard,
    halfmove_clock: u8,
    fullmove_number: u16
}

impl Default for Board {
    ///Note: This function is implemented by parsing a FEN string, which could be expensive.
    fn default() -> Self {
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".parse().unwrap()
    }
}

impl Board {
    /// Get a [`BitBoard`] of all the pieces of a certain type
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let board = Board::default();
    /// let pawns = board.pieces(Piece::Pawn);
    /// assert_eq!(pawns, bitboard! {
    ///     . . . . . . . .
    ///     X X X X X X X X
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     X X X X X X X X
    ///     . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub fn pieces(&self, piece: Piece) -> BitBoard {
        self.inner.pieces(piece)
    }

    /// Get a [`BitBoard`] of all the pieces of a certain color
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let board = Board::default();
    /// let white_pieces = board.colors(Color::White);
    /// assert_eq!(white_pieces, bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     X X X X X X X X
    ///     X X X X X X X X
    /// });
    /// ```
    #[inline(always)]
    pub fn colors(&self, color: Color) -> BitBoard {
        self.inner.colors(color)
    }

    /// Get a [`BitBoard`] of all the pieces on the board
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let board = Board::default();
    /// assert_eq!(board.occupied(), bitboard! {
    ///     X X X X X X X X
    ///     X X X X X X X X
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     X X X X X X X X
    ///     X X X X X X X X
    /// });
    /// ```
    #[inline(always)]
    pub fn occupied(&self) -> BitBoard {
        self.inner.colors(Color::White) | self.inner.colors(Color::Black)
    }

    /// Get the current side to move.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut board = Board::default();
    /// assert_eq!(board.side_to_move(), Color::White);
    /// board.play_unchecked("e2e4".parse().unwrap());
    /// assert_eq!(board.side_to_move(), Color::Black);
    /// ```
    #[inline(always)]
    pub fn side_to_move(&self) -> Color {
        self.inner.side_to_move()
    }

    /// Get the [CastleRights] for some side.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut board = Board::default();
    /// let rights = board.castle_rights(Color::White);
    /// assert_eq!(rights.short, Some(File::H));
    /// assert_eq!(rights.long, Some(File::A));
    /// board.play_unchecked("e2e4".parse().unwrap());
    /// board.play_unchecked("e7e5".parse().unwrap());
    /// board.play_unchecked("e1e2".parse().unwrap());
    /// let rights = board.castle_rights(Color::White);
    /// assert_eq!(rights.short, None);
    /// assert_eq!(rights.long, None);
    /// ```
    #[inline(always)]
    pub fn castle_rights(&self, color: Color) -> &CastleRights {
        self.inner.castle_rights(color)
    }

    #[inline(always)]
    pub fn en_passant(&self) -> Option<File> {
        self.inner.en_passant()
    }

    /// Get the incrementally updated position hash.
    /// Does not include the halfmove clock or fullmove number.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut board = Board::default();
    /// board.play_unchecked("e2e4".parse().unwrap());
    /// board.play_unchecked("e7e5".parse().unwrap());
    /// board.play_unchecked("e1e2".parse().unwrap());
    /// board.play_unchecked("e8e7".parse().unwrap());
    /// let expected: Board = "rnbq1bnr/ppppkppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR w - - 2 3"
    ///    .parse().unwrap();
    /// assert_eq!(expected.hash(), board.hash());
    /// ```
    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.inner.hash()
    }

    /// Get the pinned pieces for the side to move.
    /// Note that this counts pieces regardless of color.
    /// This counts any piece preventing check on our king.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let board: Board = "8/8/1q4k1/5p2/1n6/3B4/1KP3r1/8 w - - 0 1".parse().unwrap();
    /// assert_eq!(board.pinned(), bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . X . . . . . .
    ///     . . . . . . . .
    ///     . . X . . . . .
    ///     . . . . . . . .
    /// });
    /// ```
    #[inline(always)]
    pub fn pinned(&self) -> BitBoard {
        self.pinned
    }

    /// Get the pieces currently giving check.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut board: Board = "1r4r1/pbpknp1p/1b3P2/8/8/B1PB1q2/P4PPP/3R2K1 w - - 0 22"
    ///     .parse().unwrap();
    /// assert_eq!(board.checkers(), BitBoard::EMPTY);
    /// board.play_unchecked("d3f5".parse().unwrap());
    /// assert_eq!(board.checkers(), bitboard! {
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . X . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . . . . . .
    ///     . . . X . . . .
    /// });
    #[inline(always)]
    pub fn checkers(&self) -> BitBoard {
        self.checkers
    }

    /// Get the [halfmove clock](https://www.chessprogramming.org/Halfmove_Clock).
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut board = Board::default();
    /// assert_eq!(board.halfmove_clock(), 0);
    /// board.play_unchecked("e2e4".parse().unwrap());
    /// board.play_unchecked("e7e5".parse().unwrap());
    /// //Remains at zero for pawn moves
    /// assert_eq!(board.halfmove_clock(), 0);
    /// board.play_unchecked("e1e2".parse().unwrap());
    /// //Non-pawn move
    /// assert_eq!(board.halfmove_clock(), 1);
    /// ```
    #[inline(always)]
    pub fn halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    /// Get the [fullmove number](https://www.chessprogramming.org/Forsyth-Edwards_Notation#Fullmove_counter).
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut board = Board::default();
    /// //The fullmove number starts at one.
    /// assert_eq!(board.fullmove_number(), 1);
    /// board.play_unchecked("e2e4".parse().unwrap());
    /// board.play_unchecked("e7e5".parse().unwrap());
    /// board.play_unchecked("e1e2".parse().unwrap());
    /// //3 plies is 1.5 moves, which rounds down
    /// assert_eq!(board.fullmove_number(), 2);
    /// ```
    #[inline(always)]
    pub fn fullmove_number(&self) -> u16 {
        self.fullmove_number
    }

    /// Get the [Piece] on `square`, if there is one.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let board = Board::default();
    /// assert_eq!(board.piece_on(Square::E1), Some(Piece::King));
    /// ```
    #[inline(always)]
    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        Piece::ALL.iter().copied().find(|&p| self.pieces(p).has(square))
    }

    /// Get the [Color] of the piece on `square`, if there is one.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let board = Board::default();
    /// assert_eq!(board.color_on(Square::E1), Some(Color::White));
    /// ```
    #[inline(always)]
    pub fn color_on(&self, square: Square) -> Option<Color> {
        if self.colors(Color::White).has(square) {
            Some(Color::White)
        } else if self.colors(Color::Black).has(square) {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// Get the king square of some side. Panic if there is no king.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let board = Board::default();
    /// assert_eq!(board.king(Color::White), Square::E1);
    /// ```
    #[inline(always)]
    pub fn king(&self, color: Color) -> Square {
        (self.pieces(Piece::King) & self.colors(color)).next_square().unwrap()
    }

    /// Get the status of the game.
    /// Note that this game may still be drawn from threefold repetition.
    /// If the game is won, the loser is the current side to move.
    /// # Examples
    /// ## Checkmate
    /// ```
    /// # use cozy_chess::*;
    /// let mut board = Board::default();
    /// const MOVES: &[&str] = &[
    ///     "e2e4", "e7e5", "g1f3", "b8c6", "d2d4", "e5d4",
    ///     "f3d4", "f8c5", "c2c3", "d8f6", "d4c6", "f6f2"
    /// ];
    /// for mv in MOVES {
    ///     assert_eq!(board.status(), GameStatus::Ongoing);
    ///     board.play_unchecked(mv.parse().unwrap());
    /// }
    /// assert_eq!(board.status(), GameStatus::Won);
    /// let winner = !board.side_to_move();
    /// assert_eq!(winner, Color::Black);
    /// ```
    /// ## Stalemate
    /// ```
    /// # use cozy_chess::*;
    /// let mut board = Board::default();
    /// const MOVES: &[&str] = &[
    ///     "c2c4", "h7h5", "h2h4", "a7a5", "d1a4",
    ///     "a8a6", "a4a5", "a6h6", "a5c7", "f7f6",
    ///     "c7d7", "e8f7", "d7b7", "d8d3", "b7b8",
    ///     "d3h7", "b8c8", "f7g6", "c8e6"
    /// ];
    /// for mv in MOVES {
    ///     assert_eq!(board.status(), GameStatus::Ongoing);
    ///     board.play_unchecked(mv.parse().unwrap());
    /// }
    /// assert_eq!(board.status(), GameStatus::Drawn);
    /// ```
    /// ## 50 move rule
    /// ```
    /// # use cozy_chess::*;
    /// let mut board = Board::default();
    /// board.play_unchecked("e2e4".parse().unwrap());
    /// board.play_unchecked("e7e5".parse().unwrap());
    /// const MOVES: &[&str] = &["e1e2", "e8e7", "e2e1", "e7e8"];
    /// for mv in MOVES.iter().cycle().take(50 * 2) {
    ///     assert_eq!(board.status(), GameStatus::Ongoing);
    ///     board.play_unchecked(mv.parse().unwrap());
    /// }
    /// assert_eq!(board.status(), GameStatus::Drawn);
    /// ```
    pub fn status(&self) -> GameStatus {
        if self.halfmove_clock() >= 100 {
            GameStatus::Drawn
        } else if self.generate_moves(&mut |_| true) {
            GameStatus::Ongoing
        } else if self.checkers().empty() {
            GameStatus::Drawn
        } else {
            GameStatus::Won
        }
    }

    /// Attempt to play a [null move](https://www.chessprogramming.org/Null_Move),
    /// returning a new board if successful.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut board = Board::default();
    /// board.play_unchecked("f2f4".parse().unwrap());
    /// board.play_unchecked("e7e5".parse().unwrap());
    /// assert_eq!(board.side_to_move(), Color::White);
    /// board = board.null_move().unwrap();
    /// assert_eq!(board.side_to_move(), Color::Black);
    /// board.play_unchecked("d8h4".parse().unwrap());
    /// //Can't leave the king in check
    /// assert!(board.null_move().is_none());
    /// ```
    pub fn null_move(&self) -> Option<Board> {
        if self.checkers.empty() {
            let mut board = self.clone();
            board.halfmove_clock += 1;
            if board.side_to_move() == Color::Black {
                board.fullmove_number += 1;
            }
            board.inner.toggle_side_to_move();
            board.inner.set_en_passant(None);

            board.pinned = BitBoard::EMPTY;
            let color = board.side_to_move();
            let our_king = board.king(color);
            let their_attackers = board.colors(!color) & (
                (get_bishop_rays(our_king) & (
                    board.pieces(Piece::Bishop) |
                    board.pieces(Piece::Queen)
                )) |
                (get_rook_rays(our_king) & (
                    board.pieces(Piece::Rook) |
                    board.pieces(Piece::Queen)
                ))
            );
    
            for square in their_attackers {
                let between = get_between_rays(square, our_king) & board.occupied();
                if between.popcnt() == 1 {
                    board.pinned |= between;
                }
            }
            Some(board)
        } else {
            None
        }
    }

    /// Play a move without checking its legality. Note that this only supports Chess960 style castling.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut board = Board::default();
    /// board.play_unchecked("e2e4".parse().unwrap());
    /// board.play_unchecked("e7e5".parse().unwrap());
    /// board.play_unchecked("e1e2".parse().unwrap());
    /// board.play_unchecked("e8e7".parse().unwrap());
    /// const EXPECTED: &str = "rnbq1bnr/ppppkppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR w - - 2 3";
    /// assert_eq!(format!("{}", board), EXPECTED);
    /// ```
    pub fn play_unchecked(&mut self, mv: Move) {
        self.pinned = BitBoard::EMPTY;
        self.checkers = BitBoard::EMPTY;

        let moved = self.piece_on(mv.from).unwrap();
        let victim = self.piece_on(mv.to);
        let color = self.inner.side_to_move();
        let from_bb = mv.from.bitboard();
        let to_bb = mv.to.bitboard();
        let their_king = self.king(!color);
        let our_back_rank = Rank::First.relative_to(color);
        let their_back_rank = Rank::First.relative_to(!color);
        //Castling move encoded as king captures rook.
        let is_castle = (self.colors(color) & (from_bb ^ to_bb)).popcnt() == 2;
        let mut new_en_passant = None;

        if moved == Piece::Pawn || (victim.is_some() && !is_castle) {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
        if color == Color::Black {
            self.fullmove_number += 1;
        }

        //Lift the piece
        self.inner.xor_square(moved, color, mv.from);
        if is_castle {
            //Lift the rook too
            self.inner.xor_square(Piece::Rook, color, mv.to);

            let (king, rook) = if mv.from.file() < mv.to.file() {
                //Short castle
                (File::G, File::F)
            } else {
                //Long castle
                (File::C, File::D)
            };
            //Drop in all the pieces.
            self.inner.xor_square(Piece::King, color, Square::new(king, our_back_rank));
            self.inner.xor_square(Piece::Rook, color, Square::new(rook, our_back_rank));
            self.inner.set_castle_right(color, true, None);
            self.inner.set_castle_right(color, false, None);
        } else {
            //Drop the piece
            self.inner.xor_square(moved, color, mv.to);
            if let Some(victim) = victim {
                //If victim == piece, the piece was XORed out and this puts it back.
                //If victim != piece, the victim is still there and this XORs it out.
                self.inner.xor_square(victim, !color, mv.to);
                if mv.to.rank() == their_back_rank {
                    let rights = self.inner.castle_rights(!color);
                    if Some(mv.to.file()) == rights.short {
                        self.inner.set_castle_right(!color, true, None);
                    } else if Some(mv.to.file()) == rights.long {
                        self.inner.set_castle_right(!color, false, None);
                    }
                }
            }
            //Update checker information
            match moved {
                Piece::Knight => self.checkers |= get_knight_moves(their_king) & to_bb,
                Piece::Pawn => {
                    const PAWN_DOUBLE_MOVE_FROM: BitBoard = BitBoard(
                        Rank::Second.bitboard().0 | Rank::Seventh.bitboard().0
                    );
                    const PAWN_DOUBLE_MOVE_TO: BitBoard = BitBoard(
                        Rank::Fourth.bitboard().0 | Rank::Fifth.bitboard().0
                    );
                    if let Some(promotion) = mv.promotion {
                        //Get rid of the pawn and replace it with the promotion. Also update checkers.
                        self.inner.xor_square(Piece::Pawn, color, mv.to);
                        self.inner.xor_square(promotion, color, mv.to);
                        if promotion == Piece::Knight {
                            self.checkers |= get_knight_moves(their_king) & to_bb;
                        }
                    } else {
                        let en_passant = self.inner.en_passant().map(|ep| {
                            Square::new(ep, Rank::Third.relative_to(!color))
                        });
                        if !(from_bb & PAWN_DOUBLE_MOVE_FROM).empty()
                            && !(to_bb & PAWN_DOUBLE_MOVE_TO).empty() {
                            //Double move, update en passant.
                            new_en_passant = Some(mv.to.file());
                        } else if Some(mv.to) == en_passant {
                            //En passant capture.
                            let victim_square = Square::new(
                                mv.to.file(),
                                Rank::Fourth.relative_to(!color)
                            );
                            self.inner.xor_square(Piece::Pawn, !color, victim_square);
                        }
                        //Update checkers.
                        self.checkers |= get_pawn_attacks(their_king, !color) & to_bb;
                    }
                }
                Piece::King => {
                    self.inner.set_castle_right(color, true, None);
                    self.inner.set_castle_right(color, false, None);
                },
                Piece::Rook => if mv.from.rank() == our_back_rank {
                    let rights = self.inner.castle_rights(color);
                    if Some(mv.from.file()) == rights.short {
                        self.inner.set_castle_right(color, true, None);
                    } else if Some(mv.from.file()) == rights.long {
                        self.inner.set_castle_right(color, false, None);
                    }
                }
                _ => {}
            }
        }
        
        //Almost there. Just have to update checker and pinned information for sliding pieces.
        let our_attackers = self.colors(color) & (
            (get_bishop_rays(their_king) & (
                self.pieces(Piece::Bishop) |
                self.pieces(Piece::Queen)
            )) |
            (get_rook_rays(their_king) & (
                self.pieces(Piece::Rook) |
                self.pieces(Piece::Queen)
            ))
        );

        for square in our_attackers {
            let between = get_between_rays(square, their_king) & self.occupied();
            match between.popcnt() {
                0 => self.checkers |= square.bitboard(),
                1 => self.pinned |= between,
                _ => {}
            }
        }
        self.inner.set_en_passant(new_en_passant);
        self.inner.toggle_side_to_move();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_moves() {
        let mut board = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
            .parse::<Board>().unwrap();
        const MOVES: &[(&str, &str)] = &[
            ("f3f5", "r3k2r/p1ppqpb1/bn2pnp1/3PNQ2/1p2P3/2N4p/PPPBBPPP/R3K2R b KQkq - 1 1"),
            ("h3g2", "r3k2r/p1ppqpb1/bn2pnp1/3PNQ2/1p2P3/2N5/PPPBBPpP/R3K2R w KQkq - 0 2"),
            ("e5g6", "r3k2r/p1ppqpb1/bn2pnN1/3P1Q2/1p2P3/2N5/PPPBBPpP/R3K2R b KQkq - 0 2"),
            ("g2h1r", "r3k2r/p1ppqpb1/bn2pnN1/3P1Q2/1p2P3/2N5/PPPBBP1P/R3K2r w Qkq - 0 3"),
            ("e2f1", "r3k2r/p1ppqpb1/bn2pnN1/3P1Q2/1p2P3/2N5/PPPB1P1P/R3KB1r b Qkq - 1 3"),
            ("f7g6", "r3k2r/p1ppq1b1/bn2pnp1/3P1Q2/1p2P3/2N5/PPPB1P1P/R3KB1r w Qkq - 0 4"),
            ("d2h6", "r3k2r/p1ppq1b1/bn2pnpB/3P1Q2/1p2P3/2N5/PPP2P1P/R3KB1r b Qkq - 1 4"),
            ("e7d6", "r3k2r/p1pp2b1/bn1qpnpB/3P1Q2/1p2P3/2N5/PPP2P1P/R3KB1r w Qkq - 2 5"),
            ("f2f4", "r3k2r/p1pp2b1/bn1qpnpB/3P1Q2/1p2PP2/2N5/PPP4P/R3KB1r b Qkq f3 0 5"),
            ("e8a8", "2kr3r/p1pp2b1/bn1qpnpB/3P1Q2/1p2PP2/2N5/PPP4P/R3KB1r w Q - 1 6"),
            ("f5h5", "2kr3r/p1pp2b1/bn1qpnpB/3P3Q/1p2PP2/2N5/PPP4P/R3KB1r b Q - 2 6"),
            ("f6e4", "2kr3r/p1pp2b1/bn1qp1pB/3P3Q/1p2nP2/2N5/PPP4P/R3KB1r w Q - 0 7"),
            ("a2a4", "2kr3r/p1pp2b1/bn1qp1pB/3P3Q/Pp2nP2/2N5/1PP4P/R3KB1r b Q a3 0 7"),
            ("b4a3", "2kr3r/p1pp2b1/bn1qp1pB/3P3Q/4nP2/p1N5/1PP4P/R3KB1r w Q - 0 8"),
            ("c3d1", "2kr3r/p1pp2b1/bn1qp1pB/3P3Q/4nP2/p7/1PP4P/R2NKB1r b Q - 1 8"),
            ("a6b5", "2kr3r/p1pp2b1/1n1qp1pB/1b1P3Q/4nP2/p7/1PP4P/R2NKB1r w Q - 2 9"),
            ("h6g7", "2kr3r/p1pp2B1/1n1qp1p1/1b1P3Q/4nP2/p7/1PP4P/R2NKB1r b Q - 0 9"),
            ("d6d5", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/4nP2/p7/1PP4P/R2NKB1r w Q - 0 10"),
            ("b2b4", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P2nP2/p7/2P4P/R2NKB1r b Q b3 0 10"),
            ("e4d2", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P3P2/p7/2Pn3P/R2NKB1r w Q - 1 11"),
            ("a1b1", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P3P2/p7/2Pn3P/1R1NKB1r b - - 2 11"),
            ("h1h2", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P3P2/p7/2Pn3r/1R1NKB2 w - - 0 12"),
            ("b1c1", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P3P2/p7/2Pn3r/2RNKB2 b - - 1 12"),
            ("d2b3", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P3P2/pn6/2P4r/2RNKB2 w - - 2 13"),
            ("d1b2", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P3P2/pn6/1NP4r/2R1KB2 b - - 3 13"),
            ("c7c6", "2kr3r/p2p2B1/1np1p1p1/1b1q3Q/1P3P2/pn6/1NP4r/2R1KB2 w - - 0 14"),
            ("h5h6", "2kr3r/p2p2B1/1np1p1pQ/1b1q4/1P3P2/pn6/1NP4r/2R1KB2 b - - 1 14"),
            ("d5d6", "2kr3r/p2p2B1/1npqp1pQ/1b6/1P3P2/pn6/1NP4r/2R1KB2 w - - 2 15"),
            ("h6h2", "2kr3r/p2p2B1/1npqp1p1/1b6/1P3P2/pn6/1NP4Q/2R1KB2 b - - 0 15"),
            ("d6d1", "2kr3r/p2p2B1/1np1p1p1/1b6/1P3P2/pn6/1NP4Q/2RqKB2 w - - 1 16"),
            ("e1d1", "2kr3r/p2p2B1/1np1p1p1/1b6/1P3P2/pn6/1NP4Q/2RK1B2 b - - 0 16"),
            ("d7d6", "2kr3r/p5B1/1nppp1p1/1b6/1P3P2/pn6/1NP4Q/2RK1B2 w - - 0 17")
        ];
        for &(mv, expected) in MOVES {
            board.play_unchecked(mv.parse().unwrap());
            println!("{}, {}", mv, board.hash());
            assert_eq!(format!("{}", board), expected);
            assert_eq!(board.hash(), expected.parse::<Board>().unwrap().hash());
        }
    }
}

