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

## A note on CPU features and performance
By default, Rust binaries target a baseline CPU to ensure maximum compatibility at the cost of performance. `cozy-chess` benefits significantly from features present in modern CPUs. For maximum performance, the target CPU can instead be set to `native` to use features supported by the machine running the build. Alternatively, the target CPU can be set to `x86-64-v3`, which will produce binaries that run on most modern CPUs. The target CPU may be changed by adding `-C target-cpu=<CPU>` to `RUSTFLAGS`.

PEXT bitboards are a faster variant of the magic bitboard algorithm used by `cozy-chess`. PEXT bitboards rely on an intrinsic introduced in the [BMI2 CPU extension](https://en.wikipedia.org/wiki/X86_Bit_manipulation_instruction_set). However, it is not enabled by default, as PEXT bitboards are *slower* on AMD CPUs prior to Zen 3, which implement PEXT with microcode. PEXT bitboards can be enabled through the `pext` feature. 

## Examples
### Basic example
```rust
# use cozy_chess::*;
// Start position
let board = Board::default();
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
# use cozy_chess::*;
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

### Perft example
A [perft](https://www.chessprogramming.org/Perft) implementation exists in `examples/perft.rs`:
```text
$ cargo run --release --example perft -- 7
   Compiling cozy-chess v0.3.0
    Finished release [optimized] target(s) in 6.37s
     Running `target\release\examples\perft.exe 7`
3195901860 nodes in 10.05s (318045465 nps)
```
