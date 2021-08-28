use std::path::PathBuf;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use cozy_chess_types::*;

fn main() {
    let mut table = [BitBoard::EMPTY; 87988];
    write_moves(&mut table, &BISHOP_MAGICS, BISHOP_INDEX_BITS, &[
        SquareDelta(1, 1),
        SquareDelta(1, -1),
        SquareDelta(-1, -1),
        SquareDelta(-1, 1)
    ]);
    write_moves(&mut table, &ROOK_MAGICS, ROOK_INDEX_BITS, &[
        SquareDelta(1, 0),
        SquareDelta(0, -1),
        SquareDelta(-1, 0),
        SquareDelta(0, 1)
    ]);

    let mut out_file: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    out_file.push("sliding_moves.rs");
    let mut out_file = BufWriter::new(File::create(out_file).unwrap());
    write!(&mut out_file, "const SLIDING_MOVES: &'static [u64; {}] = &[", table.len()).unwrap();
    for magic in &table {
        write!(&mut out_file, "{},", magic.0).unwrap();
    }
    write!(&mut out_file, "];").unwrap();
}

fn write_moves(table: &mut [BitBoard; 87988], magics: &[BlackMagicEntry; Square::NUM], index_bits: usize, deltas: &[SquareDelta; 4]) {
    for &square in &Square::ALL {
        let magic = &magics[square as usize];
        let mask = !magic.mask;

        let mut blockers = BitBoard::EMPTY;
        loop {
            let mut moves = BitBoard::EMPTY;
            for delta in deltas {
                let mut square = square;
                while !blockers.has(square) {
                    if let Some(sq) = delta.add(square) {
                        square = sq;
                        moves |= square.bitboard();
                    } else {
                        break;
                    }
                }
            }
            table[get_magic_index(magics, index_bits, blockers, square)] = moves;

            //Carry-Rippler trick that enumerates all subsets of the mask, getting us all blockers.
            //https://www.chessprogramming.org/Traversing_Subsets_of_a_Set#All_Subsets_of_any_Set
            blockers = blockers.wrapping_sub(mask) & mask;
            if blockers.empty() {
                break;
            }
        }
    }
}
