#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]

use cozy_chess_types::*;

pub use color::*;
pub use piece::*;
pub use square::*;
pub use file::*;
pub use rank::*;
pub use bitboard::*;
pub use castling::*;
pub use chess_move::*;

mod board;
mod moves;

pub use board::*;
pub use moves::*;
