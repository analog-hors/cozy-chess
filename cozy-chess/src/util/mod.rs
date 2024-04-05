//! Additional common utilities

use core::fmt::Display;

use crate::*;

#[cfg(test)]
mod tests;

/// Parses a UCI move into a [`Move`].
///
/// This differs from [`Move`]'s [`core::str::FromStr`] implementation in that
/// it converts the standard UCI castling notation to the king-captures-rook
/// notation that `cozy-chess` uses (e.g. `e1g1` parses as `e1h1`).
///
/// # Examples
///
/// ```
/// # use cozy_chess::*;
/// # use cozy_chess::util::*;
/// let board: Board = "rnbqkb1r/ppp2ppp/4pn2/3p4/8/5NP1/PPPPPPBP/RNBQK2R w KQkq - 0 4"
///     .parse().unwrap();
/// assert_eq!(
///     parse_uci_move(&board, "e1g1").unwrap(),
///     "e1h1".parse::<Move>().unwrap()
/// );
/// ```
pub fn parse_uci_move(board: &Board, mv: &str) -> Result<Move, MoveParseError> {
    let mut mv: Move = mv.parse()?;

    let first_rank = Rank::First.relative_to(board.side_to_move());
    let uci_castle_start = Square::new(File::E, first_rank);
    let uci_castle_short = Square::new(File::G, first_rank);
    let uci_castle_long = Square::new(File::C, first_rank);
    let rights = board.castle_rights(board.side_to_move());

    if board.king(board.side_to_move()) == mv.from && mv.from == uci_castle_start {
        if mv.to == uci_castle_short {
            if let Some(rook_file) = rights.short {
                mv.to = Square::new(rook_file, first_rank);
            }
        }
        if mv.to == uci_castle_long {
            if let Some(rook_file) = rights.long {
                mv.to = Square::new(rook_file, first_rank);
            }
        }
    }

    Ok(mv)
}

/// Returns an object that allows printing a [`Move`] in UCI format.
///
/// This differs from [`Move`]'s [`Display`] implementation in that
/// it converts the king-captures-rook notation that `cozy-chess`
/// uses to standard UCI castling (e.g. `e1h1` displays as `e1g1`).
/// 
/// # Examples
///
/// ```
/// # use cozy_chess::*;
/// # use cozy_chess::util::*;
/// let board: Board = "rnbqkb1r/ppp2ppp/4pn2/3p4/8/5NP1/PPPPPPBP/RNBQK2R w KQkq - 0 4"
///     .parse().unwrap();
/// let castle: Move = "e1h1".parse().unwrap();
/// assert_eq!(format!("{}", display_uci_move(&board, castle)), "e1g1");
/// ```
pub fn display_uci_move(board: &Board, mv: Move) -> impl core::fmt::Display {
    let mut mv = mv;

    let first_rank = Rank::First.relative_to(board.side_to_move());
    let rights = board.castle_rights(board.side_to_move());
    let frc_castle_short = rights.short.map(|f| Square::new(f, first_rank));
    let frc_castle_long = rights.long.map(|f| Square::new(f, first_rank));

    if board.king(board.side_to_move()) == mv.from {
        if Some(mv.to) == frc_castle_short {
            mv.to = Square::new(File::G, first_rank);
        }
        if Some(mv.to) == frc_castle_long {
            mv.to = Square::new(File::C, first_rank);
        }
    }

    mv
}

/// Parses a Standard Algebraic Notation move into a [`Move`].
///
/// Canonical SAN is guaranteed to parse correctly, but non-canonical SAN may or may not parse.
/// The returned move is always legal.
///
/// # Examples
///
/// ```
/// # use cozy_chess::*;
/// # use cozy_chess::util::*;
/// let board: Board = "3k2n1/7P/Q3p3/4BPp1/Q1Q4q/8/5B2/R3K2R w KQ g6 0 1"
///     .parse().unwrap();
/// let mv: Move = "h7g8r".parse().unwrap();
/// assert_eq!(parse_san_move(&board, "hxg8=R").unwrap(), mv);
/// let mv: Move = "e1a1".parse().unwrap();
/// assert_eq!(parse_san_move(&board, "O-O-O+").unwrap(), mv);
/// let mv: Move = "e5d4".parse().unwrap();
/// assert_eq!(parse_san_move(&board, "Bd4").unwrap(), mv);
/// ```
pub fn parse_san_move(board: &Board, mv: &str) -> Result<Move, MoveParseError> {
    // SAN is easier to parse backwards
    let mut chars = mv.chars().rev().peekable();

    // Ignore check/checkmate character, we don't need it
    chars.next_if(|&c| c == '+' || c == '#');

    let dst;
    let src_rank: Option<Rank>;
    let src_file: Option<File>;
    let piece;
    let promotion;

    if chars.next_if_eq(&'O').is_some() {
        // Castles

        chars.next_if_eq(&'-').ok_or(MoveParseError)?;
        chars.next_if_eq(&'O').ok_or(MoveParseError)?;

        let rook_file = if chars.next_if_eq(&'-').is_some() {
            chars.next_if_eq(&'O').ok_or(MoveParseError)?;
            board.castle_rights(board.side_to_move()).long
        } else {
            board.castle_rights(board.side_to_move()).short
        };

        dst = Square::new(
            rook_file.ok_or(MoveParseError)?,
            board.king(board.side_to_move()).rank(),
        );
        piece = Piece::King;
        src_file = None;
        src_rank = None;
        promotion = None;
    } else {
        // Non-castles

        promotion = chars
            .peek()
            .filter(|&c| c.is_ascii_uppercase())
            .and_then(|&c| c.to_ascii_lowercase().try_into().ok());

        if promotion.is_some() {
            // Consume promotion character
            chars.next();
            // Consume optional '='
            chars.next_if_eq(&'=');
        }
        // Destination square
        let dst_rank = chars
            .next()
            .and_then(|c| c.try_into().ok())
            .ok_or(MoveParseError)?;
        let dst_file = chars
            .next()
            .and_then(|c| c.try_into().ok())
            .ok_or(MoveParseError)?;
        dst = Square::new(dst_file, dst_rank);

        // Consume optional captures
        chars.next_if_eq(&'x');

        // Source square
        src_rank = chars.peek().and_then(|&c| c.try_into().ok());
        if src_rank.is_some() {
            chars.next();
        }
        src_file = chars.peek().and_then(|&c| c.try_into().ok());
        if src_file.is_some() {
            chars.next();
        }

        // Piece
        piece = chars.next().map_or(Ok(Piece::Pawn), |c| {
            c.is_ascii_uppercase()
                .then_some(c.to_ascii_lowercase())
                .ok_or(MoveParseError)?
                .try_into()
                .map_err(|_| MoveParseError)
        })?;
    }

    if chars.next().is_some() {
        // too many characters
        return Err(MoveParseError);
    }

    let mut src_mask = board.colored_pieces(board.side_to_move(), piece);
    if let Some(src_rank) = src_rank {
        src_mask &= src_rank.bitboard();
    }
    if let Some(src_file) = src_file {
        src_mask &= src_file.bitboard();
    }

    let mut mv = None;
    board.generate_moves_for(src_mask, |mut mvs| {
        mvs.to &= dst.bitboard();
        for m in mvs {
            if m.promotion != promotion {
                continue;
            }
            if mv.is_some() {
                // ambiguous; error out
                mv = None;
                return true;
            }
            mv = Some(m);
        }
        false
    });

    mv.ok_or(MoveParseError)
}

/// Returns an object that allows printing a [`Move`] in Standard Algebraic Notation.
///
/// # Panics
/// This is guaranteed to panic if the move is illegal.
///
/// # Examples
///
/// ```
/// # use cozy_chess::*;
/// # use cozy_chess::util::*;
/// let board: Board = "3k2n1/7P/Q3p3/4BPp1/Q1Q4q/8/5B2/R3K2R w KQ g6 0 1"
///     .parse().unwrap();
/// let mv: Move = "h7g8r".parse().unwrap();
/// assert_eq!(format!("{}", display_san_move(&board, mv)), "hxg8=R+");
/// let mv: Move = "e1a1".parse().unwrap();
/// assert_eq!(format!("{}", display_san_move(&board, mv)), "O-O-O+");
/// let mv: Move = "e5d4".parse().unwrap();
/// assert_eq!(format!("{}", display_san_move(&board, mv)), "Bd4");
/// ```
pub fn display_san_move(board: &Board, mv: Move) -> impl Display {
    let mut after_board = board.clone();
    after_board.play(mv);

    let check = !after_board.checkers().is_empty();
    let checkmate = check && !after_board.generate_moves(|_| true);

    let piece = board.piece_on(mv.from).unwrap();
    let captures = board.occupied().len() > after_board.occupied().len();

    let first_rank = Rank::First.relative_to(board.side_to_move());
    let rights = board.castle_rights(board.side_to_move());
    let castle_short = rights.short.map(|f| Square::new(f, first_rank));
    let castle_long = rights.long.map(|f| Square::new(f, first_rank));

    if piece == Piece::King && Some(mv.to) == castle_short || Some(mv.to) == castle_long {
        return SanDisplay {
            piece: None,
            from_file: None,
            from_rank: None,
            captures,
            to_sq: mv.to,
            promotion: None,
            check,
            checkmate,
            long_castles: Some(mv.to) == castle_long,
            short_castles: Some(mv.to) == castle_short,
        };
    }

    let mut file_disambiguates = true;
    let mut rank_disambiguates = true;
    let mut ambiguous = false;

    board.generate_moves_for(board.colored_pieces(board.side_to_move(), piece), |mvs| {
        if mvs.from != mv.from && mvs.to.has(mv.to) {
            ambiguous = true;
            if mvs.from.file() == mv.from.file() {
                file_disambiguates = false;
            }
            if mvs.from.rank() == mv.from.rank() {
                rank_disambiguates = false;
            }
        }
        false
    });

    if piece == Piece::Pawn && captures {
        ambiguous = true;
    }

    let (from_file, from_rank) = match (ambiguous, file_disambiguates, rank_disambiguates) {
        (false, _, _) => (None, None),
        (true, true, _) => (Some(mv.from.file()), None),
        (true, false, false) => (Some(mv.from.file()), Some(mv.from.rank())),
        (true, false, true) => (None, Some(mv.from.rank())),
    };

    SanDisplay {
        piece: (piece != Piece::Pawn).then_some(piece),
        from_file,
        from_rank,
        captures,
        to_sq: mv.to,
        promotion: mv.promotion,
        check,
        checkmate,
        long_castles: false,
        short_castles: false,
    }
}

struct SanDisplay {
    piece: Option<Piece>,
    from_file: Option<File>,
    from_rank: Option<Rank>,
    captures: bool,
    to_sq: Square,
    promotion: Option<Piece>,
    check: bool,
    checkmate: bool,
    long_castles: bool,
    short_castles: bool,
}

impl Display for SanDisplay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.long_castles {
            write!(f, "O-O-O")?;
        } else if self.short_castles {
            write!(f, "O-O")?;
        } else {
            if let Some(piece) = self.piece {
                write!(f, "{}", char::to_ascii_uppercase(&piece.into()))?;
            }
            if let Some(file) = self.from_file {
                write!(f, "{file}")?;
            }
            if let Some(rank) = self.from_rank {
                write!(f, "{rank}")?;
            }
            if self.captures {
                write!(f, "x")?;
            }
            write!(f, "{}", self.to_sq)?;
            if let Some(promo) = self.promotion {
                write!(f, "={}", char::to_ascii_uppercase(&promo.into()))?;
            }
        }

        if self.checkmate {
            write!(f, "#")?;
        } else if self.check {
            write!(f, "+")?;
        }

        Ok(())
    }
}
