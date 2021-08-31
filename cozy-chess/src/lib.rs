//! # `cozy-chess`

//! ## Rust Chess and Chess960 move generation library
//! `cozy-chess` is a Chess and Chess960 move generation library written in Rust.
//! It is largely inspired by Jordan Bray's neat [`chess`](https://github.com/jordanbray/chess)
//! move generation library.
//! See the [`Board`] struct to get started.

use cozy_chess_types::*;

pub use color::*;
pub use piece::*;
pub use square::*;
pub use file::*;
pub use rank::*;
pub use bitboard::*;
pub use castling::*;
pub use chess_move::*;

pub mod board;
pub mod moves;

pub use board::*;
pub use moves::*;
