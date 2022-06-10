# `cozy-chess`

## Rust Chess and Chess960 move generation library
[![crates.io](https://img.shields.io/crates/v/cozy-chess.svg)](https://crates.io/crates/cozy-chess)

`cozy-chess` is a Chess and Chess960 (Fischer Random Chess) move generation library written in Rust that aims to provide competitive move generation performance. It is largely inspired by Jordan Bray's neat [`chess`](https://github.com/jordanbray/chess) library. `cozy-chess` aims to be a safer alternative to `chess` that maintains correctness while providing similar performance.

<img src="https://static.manebooru.art/img/view/2020/10/8/1827770.jpg" alt="Cozy Glow" width=20% height=30%>

## Features
- Supports Chess, Chess960/FRC, and Double Chess960/DFRC
- Strongly-typed API that makes heavy use of newtypes to avoid errors
- Performant legal move generation suitable for use in a chess engine
    - Implements fixed shift fancy black magic bitboards
    - Optionally implements PEXT bitboards based on the BMI2 intrinsic
    - Flexible API produces moves in bulk for optional bulk filtering
- Efficient bitboard-based board representation
- Incrementally updated zobrist hash for quickly obtaining a hash of a board

## Examples
### Basic example
```rust
// Start position
let board = Board::default();
// Likely a fixed sized stack vector in a real engine
let mut move_list = Vec::new();
board.generate_moves(|moves| {
    // Unpack dense move set into move list
    move_list.extend(moves);
    false
});
assert_eq!(move_list.len(), 20);
```

### Get capture moves in bulk
```rust
// Parse position from FEN
let board = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
    .parse::<Board>()
    .unwrap();

let mut total_moves = 0;
let mut total_captures = 0;

let enemy_pieces = board.colors(!board.side_to_move());
board.generate_moves(|moves| {
    let mut captures = moves.clone();
    // Bitmask to efficiently get all captures set-wise.
    // Excluding en passant square for convenience.
    captures.to &= enemy_pieces;

    total_moves += moves.len();
    total_captures += captures.len();
    false
});

assert_eq!(total_moves, 48);
assert_eq!(total_captures, 8);
```

### Number of possible chess games after N plies (half moves)
```rust
fn perft(board: &Board, plies: u32) -> u64 {
    if plies == 0 {
        return 1;
    }

    let mut leaf_nodes = 0;
    board.generate_moves(|moves| {
        for mv in moves {
            let mut child = board.clone();
            child.play_unchecked(mv);
            leaf_nodes += perft(&child, plies - 1);
        }
        false
    });
    leaf_nodes
}

let mut board = Board::default();

assert_eq!(perft(&board, 1), 20);
assert_eq!(perft(&board, 2), 400);
assert_eq!(perft(&board, 3), 8902);
```
