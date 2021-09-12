use std::convert::TryInto;
use std::str::FromStr;
use std::fmt::{Display, Formatter};

use crate::*;

use super::ZobristBoard;

impl Board {
    /// Check if the board is valid. If not, other functions may not work as expected.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut board = Board::default();
    /// assert!(board.validity_check());
    /// let _ = board.try_play_unchecked("e1e8".parse().unwrap());
    /// assert!(!board.validity_check());
    /// ```
    pub fn validity_check(&self) -> bool {
        macro_rules! soft_assert {
            ($expr:expr) => {
                if !$expr {
                    return false;
                }
            }
        }

        //Verify that the board's data makes sense. The bitboards should not overlap.
        let mut occupied = BitBoard::EMPTY;
        for piece in Piece::ALL {
            let pieces = self.pieces(piece);
            soft_assert!((pieces & occupied).empty());
            occupied |= pieces;
        }
        soft_assert!((self.colors(Color::White) & self.colors(Color::Black)).empty());
        soft_assert!(occupied == self.occupied());
        
        for &color in &Color::ALL {
            let pieces = self.colors(color);
            soft_assert!((pieces & self.pieces(Piece::King)).popcnt() == 1);
            soft_assert!(pieces.popcnt() <= 16);
            soft_assert!((pieces & self.pieces(Piece::Pawn)).popcnt() <= 8);

            let back_rank = Rank::First.relative_to(color);
            soft_assert!((pieces & self.pieces(Piece::Pawn) & back_rank.bitboard()).empty());

            let rights = self.castle_rights(color);
            let our_rooks = pieces & self.pieces(Piece::Rook);
            if rights.short.is_some() || rights.long.is_some() {
                let our_king = self.king(color);
                soft_assert!(our_king.rank() == back_rank);
                if let Some(rook) = rights.long {
                    soft_assert!(our_rooks.has(Square::new(rook, back_rank)));
                    soft_assert!(rook < our_king.file());
                }
                if let Some(rook) = rights.short {
                    soft_assert!(our_rooks.has(Square::new(rook, back_rank)));
                    soft_assert!(our_king.file() < rook);
                }
            }
        }

        let color = self.side_to_move();
        if let Some(en_passant) = self.en_passant() {
            let en_passant_square = Square::new(
                en_passant,
                Rank::Third.relative_to(!color)
            );
            let en_passant_pawn = Square::new(
                en_passant,
                Rank::Fourth.relative_to(!color)
            );
            soft_assert!(!self.occupied().has(en_passant_square));
            soft_assert!((self.colors(!color) & self.pieces(Piece::Pawn)).has(en_passant_pawn));
        }

        let (our_checkers, _) = self.calculate_checkers_and_pins(!color);
        //Opponent can't be in check while it's our turn
        soft_assert!(our_checkers.empty());

        let (checkers, pinned) = self.calculate_checkers_and_pins(color);
        soft_assert!(self.checkers() == checkers);
        soft_assert!(self.pinned() == pinned);
        soft_assert!(self.checkers().popcnt() < 3);

        true
    }

    /// Parse a FEN string. If `shredder` is true, it parses Shredder FEN instead.
    /// You can also parse the board with [`FromStr`], which parses regular FEN.
    /// # Examples
    /// ## FEN
    /// ```
    /// # use cozy_chess::*;
    /// const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let board = Board::from_fen(STARTPOS, false).unwrap();
    /// assert_eq!(format!("{}", board), STARTPOS);
    /// ```
    /// ## Shredder FEN
    /// ```
    /// # use cozy_chess::*;
    /// const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1";
    /// let board = Board::from_fen(STARTPOS, true).unwrap();
    /// assert_eq!(format!("{:#}", board), STARTPOS);
    /// ```
    pub fn from_fen(fen: &str, shredder: bool) -> Result<Self, FenParseError> {
        let mut board = Self {
            inner: ZobristBoard::empty(),
            pinned: BitBoard::EMPTY,
            checkers: BitBoard::EMPTY,
            halfmove_clock: 0,
            fullmove_number: 0
        };
        let mut parts = fen.split(' ');
        macro_rules! parse_fields {
            ($($parser:expr, $error:expr;)*) => {
                $(parts.next().and_then($parser).ok_or($error)?;)*
            }
        }
        parse_fields! {
            |s| {
                for (rank, row) in s.rsplit('/').enumerate() {
                    let rank = Rank::try_index(rank)?;
                    let mut file = 0;
                    for p in row.chars() {
                        if let Some(offset) = p.to_digit(10) {
                            file += offset as usize;
                        } else {
                            let piece = p.to_ascii_lowercase().try_into().ok()?;
                            let color = if p.is_ascii_uppercase() {
                                Color::White
                            } else {
                                Color::Black
                            };
                            let square = Square::new(
                                File::try_index(file)?,
                                rank
                            );
                            board.inner.xor_square(piece, color, square);
                            file += 1;
                        }
                    }
                    if file != File::NUM {
                        return None;
                    }
                }
                Some(())
            }, FenParseError::InvalidBoard;
            |s| {
                if s.parse::<Color>().ok()? != board.side_to_move() {
                    board.inner.toggle_side_to_move();
                }
                Some(())
            }, FenParseError::InvalidSideToMove;
            |s| {
                if s != "-" {
                    for c in s.chars() {
                        let color = if c.is_ascii_uppercase() {
                            Color::White
                        } else {
                            Color::Black
                        };
                        let king_file = (
                            board.pieces(Piece::King) &
                            board.colors(color)
                        ).next_square()?.file();
                        let (short, file) = if shredder {
                            let file = c.to_ascii_lowercase().try_into().ok()?;
                            (king_file < file, file)
                        } else {
                            match c.to_ascii_lowercase() {
                                'k' => (true, File::H),
                                'q' => (false, File::A),
                                _ => return None
                            }
                        };
                        let rights = board.castle_rights(color);
                        let prev = if short {
                            rights.short
                        } else {
                            rights.long
                        };
                        if prev.is_some() {
                            //Duplicates
                            return None;
                        }
                        board.inner.set_castle_right(color, short, Some(file));
                    }
                }
                Some(())
            }, FenParseError::InvalidCastlingRights;
            |s| {
                if s != "-" {
                    let square = s.parse::<Square>().ok()?;
                    let en_passant_rank = Rank::Third.relative_to(!board.side_to_move());
                    if square.rank() != en_passant_rank {
                        return None;
                    }
                    board.inner.set_en_passant(Some(square.file()));
                }
                Some(())
            }, FenParseError::InvalidEnPassant;
            |s| {
                board.halfmove_clock = s.parse().ok()?;
                if board.halfmove_clock > 100 {
                    return None;
                }
                Some(())
            }, FenParseError::InvalidHalfMoveClock;
            |s| {
                board.fullmove_number = s.parse().ok()?;
                if board.fullmove_number == 0 {
                    return None;
                }
                Some(())
            }, FenParseError::InvalidFullmoveNumber;
        }
        if parts.next().is_some() {
            return Err(FenParseError::TooManyFields);
        }

        let color = board.side_to_move();
        let our_pieces = board.colors(color); 
        let their_pieces = board.colors(!color);
        let our_kings = (board.pieces(Piece::King) & our_pieces).popcnt();
        let their_kings = (board.pieces(Piece::King) & their_pieces).popcnt();
        if our_kings == 1 && their_kings == 1 {
            let (checkers, pinned) = board.calculate_checkers_and_pins(color);
            board.checkers = checkers;
            board.pinned = pinned;
        }

        if !board.validity_check() {
            return Err(FenParseError::InvalidBoard);
        }

        Ok(board)
    }

    fn calculate_checkers_and_pins(&self, color: Color) -> (BitBoard, BitBoard) {
        let our_king = self.king(color);
        let their_pieces = self.colors(!color);

        let mut checkers = BitBoard::EMPTY;
        let mut pinned = BitBoard::EMPTY;

        let their_attackers = their_pieces & (
            (get_bishop_rays(our_king) & (
                self.pieces(Piece::Bishop) |
                self.pieces(Piece::Queen)
            )) |
            (get_rook_rays(our_king) & (
                self.pieces(Piece::Rook) |
                self.pieces(Piece::Queen)
            ))
        );
        for attacker in their_attackers {
            let between = get_between_rays(attacker, our_king) &
                self.occupied();
            match between.popcnt() {
                0 => checkers |= attacker.bitboard(),
                1 => pinned |= between,
                _ => {}
            }
        }

        checkers |= get_knight_moves(our_king)
            & their_pieces
            & self.pieces(Piece::Knight);
        checkers |= get_pawn_attacks(our_king, color)
            & their_pieces
            & self.pieces(Piece::Pawn);
        (checkers, pinned)
    }
}


#[derive(Debug, Clone, Copy)]
pub enum FenParseError {
    InvalidBoard,
    InvalidSideToMove,
    InvalidCastlingRights,
    InvalidEnPassant,
    InvalidHalfMoveClock,
    InvalidFullmoveNumber,
    TooManyFields
}

impl FromStr for Board {
    type Err = FenParseError;

    /// Parse the board. You can also parse Shredder FEN with [`Board::from_fen`]
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let board: Board = STARTPOS.parse().unwrap();
    /// assert_eq!(format!("{}", board), STARTPOS);
    /// ```
    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        Self::from_fen(fen, false)
    }
}

impl Display for Board {
    /// Display the board. You can use the alternate format mode for Shredder FEN
    /// # Examples
    /// ## FEN
    /// ```
    /// # use cozy_chess::*;
    /// const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// let board = Board::default();
    /// assert_eq!(format!("{}", board), STARTPOS);
    /// ```
    /// ## Shredder FEN
    /// ```
    /// # use cozy_chess::*;
    /// const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1";
    /// let board = Board::default();
    /// assert_eq!(format!("{:#}", board), STARTPOS);
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let shredder = f.alternate();
        for &rank in Rank::ALL.iter().rev() {
            let mut empty = 0;
            for &file in &File::ALL {
                let square = Square::new(file, rank);
                if let Some(piece) = self.piece_on(square) {
                    if empty > 0 {
                        write!(f, "{}", empty)?;
                        empty = 0;
                    }
                    let mut piece: char = piece.into();
                    if self.color_on(square).unwrap() == Color::White {
                        piece = piece.to_ascii_uppercase();
                    }
                    write!(f, "{}", piece)?;
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                write!(f, "{}", empty)?;
            }
            if rank > Rank::First {
                write!(f, "/")?;
            }
        }
        write!(f, " {} ", self.side_to_move())?;
        let mut wrote_castle_rights = false;
        for &color in &Color::ALL {
            let rights = self.castle_rights(color);
            let short = rights.short.map(|file| if shredder {
                file.into()
            } else {
                'k'
            });
            let long = rights.long.map(|file| if shredder {
                file.into()
            } else {
                'q'
            });
            for mut right in short.into_iter().chain(long) {
                if color == Color::White {
                    right = right.to_ascii_uppercase();
                }
                wrote_castle_rights = true;
                write!(f , "{}", right)?;
            }
        }
        if !wrote_castle_rights {
            write!(f , "-")?;
        }
        if let Some(file) = self.en_passant() {
            let rank = Rank::Third.relative_to(!self.side_to_move());
            write!(f, " {}", Square::new(file, rank))?;
        } else {
            write!(f, " -")?;
        }
        write!(f, " {} {}", self.halfmove_clock, self.fullmove_number)?;
        Ok(())
    }
}
