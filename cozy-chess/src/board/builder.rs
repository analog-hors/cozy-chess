use std::num::NonZeroU16;

use crate::*;

use super::zobrist::ZobristBoard;

/// An error while building a board.
#[derive(Debug, Clone, Copy)]
pub enum BoardBuilderError {
    InvalidBoard,
    InvalidSideToMove,
    InvalidCastlingRights,
    InvalidEnPassant,
    InvalidHalfMoveClock,
    InvalidFullmoveNumber,
}

/// A board builder to manipulate arbitrary boards.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoardBuilder {
    pub board: [Option<(Piece, Color)>; Square::NUM],
    pub side_to_move: Color,
    pub castle_rights: [CastleRights; Color::NUM],
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub fullmove_number: NonZeroU16
}

/// Note: This function is implemented by parsing a FEN string, which could be expensive.
impl Default for BoardBuilder {
    fn default() -> Self {
        BoardBuilder::from_board(&Board::default()).unwrap()
    }
}

impl BoardBuilder {
    /// Create a builder from a [`Board`].
    /// # Errors
    /// This will error (return [`None`]) if the board is invalid.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let builder = BoardBuilder::default().build().unwrap();
    /// assert_eq!(builder, Board::default());
    /// ```
    pub fn from_board(board: &Board) -> Option<Self> {
        if !board.validity_check() {
            return None;
        }
        let mut this = BoardBuilder::empty();
        for &color in &Color::ALL {
            let pieces = board.colors(color);
            for &piece in &Piece::ALL {
                let pieces = pieces & board.pieces(piece);
                for square in pieces {
                    *this.square_mut(square) = Some((piece, color));
                }
            }
            *this.castle_rights_mut(color) = *board.castle_rights(color);
        }
        this.side_to_move = board.side_to_move();
        let en_passant_rank = Rank::Third.relative_to(!board.side_to_move());
        this.en_passant = board.en_passant().map(|f| Square::new(f, en_passant_rank));
        this.halfmove_clock = board.halfmove_clock();
        this.fullmove_number = board.fullmove_number().try_into().unwrap();
        Some(this)
    }

    /// Get an empty builder. All fields are set to their empty values.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let builder = BoardBuilder::empty();
    /// for &square in &Square::ALL {
    ///     assert!(builder.square(square).is_none());
    /// }
    /// ```
    pub fn empty() -> Self {
        Self {
            board: [None; Square::NUM],
            side_to_move: Color::White,
            castle_rights: [CastleRights::EMPTY; Color::NUM],
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1.try_into().unwrap()
        }
    }

    /// Get a square on the board.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let builder = BoardBuilder::default();
    /// assert_eq!(builder.square(Square::A1), Some((Piece::Rook, Color::White)));
    /// ```
    pub fn square(&self, square: Square) -> Option<(Piece, Color)> {
        self.board[square as usize]
    }

    /// Mutably get a square on the board.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut builder = BoardBuilder::default();
    /// *builder.square_mut(Square::A1) = Some((Piece::Knight, Color::White));
    /// assert_eq!(builder.square(Square::A1), Some((Piece::Knight, Color::White)));
    /// ```
    pub fn square_mut(&mut self, square: Square) -> &mut Option<(Piece, Color)> {
        &mut self.board[square as usize]
    }

    /// Get the castle rights for a side.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let builder = BoardBuilder::default();
    /// let rights = builder.castle_rights(Color::White);
    /// assert_eq!(rights.short, Some(File::H));
    /// assert_eq!(rights.long, Some(File::A));
    /// ```
    pub fn castle_rights(&self, color: Color) -> &CastleRights {
        &self.castle_rights[color as usize]
    }

    /// Mutably get the castle rights for a side.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let mut builder = BoardBuilder::default();
    /// let rights = builder.castle_rights_mut(Color::White);
    /// rights.short = None;
    /// assert_eq!(rights.short, None);
    /// ```
    pub fn castle_rights_mut(&mut self, color: Color) -> &mut CastleRights {
        &mut self.castle_rights[color as usize]
    }

    /// Build a [`Board`] from this builder.
    /// # Errors
    /// This will error if the current state is invalid.
    /// # Examples
    /// ```
    /// # use cozy_chess::*;
    /// let builder = BoardBuilder::default().build().unwrap();
    /// assert_eq!(builder, Board::default());
    /// ```
    pub fn build(&self) -> Result<Board, BoardBuilderError> {
        use BoardBuilderError::*;

        let mut board = Board {
            inner: ZobristBoard::empty(),
            pinned: BitBoard::EMPTY,
            checkers: BitBoard::EMPTY,
            halfmove_clock: 0,
            fullmove_number: 0
        };

        self.add_board          (&mut board).map_err(|_| InvalidBoard)?;
        self.add_castle_rights  (&mut board).map_err(|_| InvalidCastlingRights)?;
        self.add_en_passant     (&mut board).map_err(|_| InvalidEnPassant)?;
        self.add_halfmove_clock (&mut board).map_err(|_| InvalidHalfMoveClock)?;
        self.add_fullmove_number(&mut board).map_err(|_| InvalidFullmoveNumber)?;
        
        let (checkers, pinned) = board.calculate_checkers_and_pins(board.side_to_move());
        board.checkers = checkers;
        board.pinned = pinned;
        Ok(board)
    }

    fn add_board(&self, board: &mut Board) -> Result<(), ()> {
        for &square in &Square::ALL {
            if let Some((piece, color)) = self.square(square) {
                board.inner.xor_square(piece, color, square);
            }
        }
        if self.side_to_move != board.side_to_move() {
            board.inner.toggle_side_to_move();
        }
        if !board.board_is_valid() {
            return Err(());
        }
        Ok(())
    }

    fn add_castle_rights(&self, board: &mut Board) -> Result<(), ()> {
        for &color in &Color::ALL {
            let rights = self.castle_rights[color as usize];
            board.inner.set_castle_right(color, true, rights.short);
            board.inner.set_castle_right(color, false, rights.long);
        }
        if !board.castle_rights_are_valid() {
            return Err(());
        }
        Ok(())
    }

    fn add_en_passant(&self, board: &mut Board) -> Result<(), ()> {
        if let Some(square) = self.en_passant {
            let en_passant_rank = Rank::Third.relative_to(!board.side_to_move());
            if square.rank() != en_passant_rank {
                return Err(());
            }
            board.inner.set_en_passant(Some(square.file()));
        }
        if !board.en_passant_is_valid() {
            return Err(());
        }
        Ok(())
    }

    fn add_halfmove_clock(&self, board: &mut Board) -> Result<(), ()> {
        if self.halfmove_clock > 100 {
            return Err(());
        }
        board.halfmove_clock = self.halfmove_clock;
        if !board.halfmove_clock_is_valid() {
            return Err(());
        }
        Ok(())
    }

    fn add_fullmove_number(&self, board: &mut Board) -> Result<(), ()> {
        board.fullmove_number = self.fullmove_number.into();
        if !board.fullmove_number_is_valid() {
            return Err(());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_board() {
        for fen in include_str!("test_data/valid.sfens").lines() {
            let board = Board::from_fen(&fen, true).unwrap();
            let builder = BoardBuilder::from_board(&board).unwrap();
            assert_eq!(builder.build().unwrap(), board);
        }
    }

    //No invalid FEN test yet due to lack of invalid FEN data.
}
