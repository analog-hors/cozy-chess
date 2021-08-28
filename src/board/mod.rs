use crate::*;

mod movegen;
mod parse;
mod zobrist;

use zobrist::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GameStatus {
    Checkmate,
    Stalemate,
    Ongoing
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    inner: ZobristBoard,
    pinned: BitBoard,
    checkers: BitBoard
}

impl Default for Board {
    ///Note: This function is implemented by parsing a FEN string, which could be expensive.
    fn default() -> Self {
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".parse().unwrap()
    }
}

impl Board {
    #[inline(always)]
    pub fn pieces(&self, piece: Piece) -> BitBoard {
        self.inner.pieces(piece)
    }

    #[inline(always)]
    pub fn colors(&self, color: Color) -> BitBoard {
        self.inner.colors(color)
    }

    #[inline(always)]
    pub fn occupied(&self) -> BitBoard {
        self.inner.colors(Color::White) | self.inner.colors(Color::Black)
    }

    #[inline(always)]
    pub fn side_to_move(&self) -> Color {
        self.inner.side_to_move()
    }

    #[inline(always)]
    pub fn castle_rights(&self, color: Color) -> &CastleRights {
        self.inner.castle_rights(color)
    }

    #[inline(always)]
    pub fn en_passant(&self) -> Option<File> {
        self.inner.en_passant()
    }

    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.inner.hash()
    }
    
    #[inline(always)]
    pub fn pinned(&self) -> BitBoard {
        self.pinned
    }

    #[inline(always)]
    pub fn checkers(&self) -> BitBoard {
        self.checkers
    }

    #[inline(always)]
    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        Piece::ALL.iter().copied().find(|&p| self.pieces(p).has(square))
    }

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

    #[inline(always)]
    pub fn king(&self, color: Color) -> Square {
        (self.pieces(Piece::King) & self.colors(color)).next_square().unwrap()
    }

    pub fn status(&self) -> GameStatus {
        if self.generate_moves(&mut |_| true) {
            GameStatus::Ongoing
        } else if self.checkers().empty() {
            GameStatus::Stalemate
        } else {
            GameStatus::Checkmate
        }
    }

    pub fn null_move(&mut self) -> bool {
        if self.checkers.empty() {
            let color = self.side_to_move();
            let their_king = self.king(color);
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
                if between.popcnt() == 1 {
                    self.pinned |= between;
                }
            }

            self.inner.toggle_side_to_move();
            self.inner.set_en_passant(None);
            true
        } else {
            false
        }
    }

    ///Play a move without checking its legality. Note that this only supports Chess960 style castling.
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
        const MOVES: &'static [(&'static str, &'static str)] = &[
            ("f3f5", "r3k2r/p1ppqpb1/bn2pnp1/3PNQ2/1p2P3/2N4p/PPPBBPPP/R3K2R b KQkq - 0 1"),
            ("h3g2", "r3k2r/p1ppqpb1/bn2pnp1/3PNQ2/1p2P3/2N5/PPPBBPpP/R3K2R w KQkq - 0 1"),
            ("e5g6", "r3k2r/p1ppqpb1/bn2pnN1/3P1Q2/1p2P3/2N5/PPPBBPpP/R3K2R b KQkq - 0 1"),
            ("g2h1r", "r3k2r/p1ppqpb1/bn2pnN1/3P1Q2/1p2P3/2N5/PPPBBP1P/R3K2r w Qkq - 0 1"),
            ("e2f1", "r3k2r/p1ppqpb1/bn2pnN1/3P1Q2/1p2P3/2N5/PPPB1P1P/R3KB1r b Qkq - 0 1"),
            ("f7g6", "r3k2r/p1ppq1b1/bn2pnp1/3P1Q2/1p2P3/2N5/PPPB1P1P/R3KB1r w Qkq - 0 1"),
            ("d2h6", "r3k2r/p1ppq1b1/bn2pnpB/3P1Q2/1p2P3/2N5/PPP2P1P/R3KB1r b Qkq - 0 1"),
            ("e7d6", "r3k2r/p1pp2b1/bn1qpnpB/3P1Q2/1p2P3/2N5/PPP2P1P/R3KB1r w Qkq - 0 1"),
            ("f2f4", "r3k2r/p1pp2b1/bn1qpnpB/3P1Q2/1p2PP2/2N5/PPP4P/R3KB1r b Qkq f3 0 1"),
            ("e8a8", "2kr3r/p1pp2b1/bn1qpnpB/3P1Q2/1p2PP2/2N5/PPP4P/R3KB1r w Q - 0 1"),
            ("f5h5", "2kr3r/p1pp2b1/bn1qpnpB/3P3Q/1p2PP2/2N5/PPP4P/R3KB1r b Q - 0 1"),
            ("f6e4", "2kr3r/p1pp2b1/bn1qp1pB/3P3Q/1p2nP2/2N5/PPP4P/R3KB1r w Q - 0 1"),
            ("a2a4", "2kr3r/p1pp2b1/bn1qp1pB/3P3Q/Pp2nP2/2N5/1PP4P/R3KB1r b Q a3 0 1"),
            ("b4a3", "2kr3r/p1pp2b1/bn1qp1pB/3P3Q/4nP2/p1N5/1PP4P/R3KB1r w Q - 0 1"),
            ("c3d1", "2kr3r/p1pp2b1/bn1qp1pB/3P3Q/4nP2/p7/1PP4P/R2NKB1r b Q - 0 1"),
            ("a6b5", "2kr3r/p1pp2b1/1n1qp1pB/1b1P3Q/4nP2/p7/1PP4P/R2NKB1r w Q - 0 1"),
            ("h6g7", "2kr3r/p1pp2B1/1n1qp1p1/1b1P3Q/4nP2/p7/1PP4P/R2NKB1r b Q - 0 1"),
            ("d6d5", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/4nP2/p7/1PP4P/R2NKB1r w Q - 0 1"),
            ("b2b4", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P2nP2/p7/2P4P/R2NKB1r b Q b3 0 1"),
            ("e4d2", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P3P2/p7/2Pn3P/R2NKB1r w Q - 0 1"),
            ("a1b1", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P3P2/p7/2Pn3P/1R1NKB1r b - - 0 1"),
            ("h1h2", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P3P2/p7/2Pn3r/1R1NKB2 w - - 0 1"),
            ("b1c1", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P3P2/p7/2Pn3r/2RNKB2 b - - 0 1"),
            ("d2b3", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P3P2/pn6/2P4r/2RNKB2 w - - 0 1"),
            ("d1b2", "2kr3r/p1pp2B1/1n2p1p1/1b1q3Q/1P3P2/pn6/1NP4r/2R1KB2 b - - 0 1"),
            ("c7c6", "2kr3r/p2p2B1/1np1p1p1/1b1q3Q/1P3P2/pn6/1NP4r/2R1KB2 w - - 0 1"),
            ("h5h6", "2kr3r/p2p2B1/1np1p1pQ/1b1q4/1P3P2/pn6/1NP4r/2R1KB2 b - - 0 1"),
            ("d5d6", "2kr3r/p2p2B1/1npqp1pQ/1b6/1P3P2/pn6/1NP4r/2R1KB2 w - - 0 1"),
            ("h6h2", "2kr3r/p2p2B1/1npqp1p1/1b6/1P3P2/pn6/1NP4Q/2R1KB2 b - - 0 1"),
            ("d6d1", "2kr3r/p2p2B1/1np1p1p1/1b6/1P3P2/pn6/1NP4Q/2RqKB2 w - - 0 1"),
            ("e1d1", "2kr3r/p2p2B1/1np1p1p1/1b6/1P3P2/pn6/1NP4Q/2RK1B2 b - - 0 1"),
            ("d7d6", "2kr3r/p5B1/1nppp1p1/1b6/1P3P2/pn6/1NP4Q/2RK1B2 w - - 0 1"),
        ];
        for &(mv, expected) in MOVES {
            board.play_unchecked(mv.parse().unwrap());
            println!("{}, {}", mv, board.hash());
            assert_eq!(format!("{}", board), expected);
            assert_eq!(board.hash(), expected.parse::<Board>().unwrap().hash());
        }
    }
}
