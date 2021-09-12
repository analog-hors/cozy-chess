//! # `cozy-chess`

//! ## Rust Chess and Chess960 move generation library

//! `cozy-chess` is a Chess and Chess960 move generation library written in Rust.
//! It is largely inspired by Jordan Bray's neat [`chess`](https://github.com/jordanbray/chess)
//! move generation library. Compared to `chess`, it provides a more ergonomic but less featureful
//! interface. It also uses a significantly lower (currently zero) amount of `unsafe`, which has
//! contributed to numerous unsoundness bugs in `chess`. This was the primary reason for its
//! creation. Basic perft testing seems to indicate similar performance to `chess`.
//! See the [`Board`] struct to get started.
//! <br>
//! <br>
//! <img src="https://static.manebooru.art/img/view/2020/10/8/1827770.jpg" alt="Cozy Glow" width=30% height=30%>

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
