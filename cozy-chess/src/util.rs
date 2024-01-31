//! Additional common utilities

use crate::*;

/// Parses a UCI move into a [`Move`]
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
