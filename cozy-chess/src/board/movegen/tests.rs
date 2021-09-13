use super::*;

fn perft(board: &Board, depth: u8) -> u64 {
    let mut nodes = 0;
    match depth {
        0 => nodes += 1,
        1 => {
            board.generate_moves(|moves| {
                nodes += moves.len() as u64;
                false
            });
        }
        _ => {
            board.generate_moves(|moves| {
                for mv in moves {
                    let mut board = board.clone();
                    board.play_unchecked(mv);
                    let child_nodes = perft(&board, depth - 1);
                    nodes += child_nodes;
                }
                false
            });
        }
    }
    nodes
}

macro_rules! make_perft_test {
    ($($name:ident($board:expr $(,$node:expr)*);)*) => {
        $(#[test]
        fn $name() {
            let board = $board.parse::<Board>()
                .or_else(|_| Board::from_fen($board, true))
                .unwrap();
            const NODES: &'static [u64] = &[$($node),*];
            for (depth, &nodes) in NODES.iter().enumerate() {
                assert_eq!(perft(&board, depth as u8), nodes, "Perft {}", depth);
            }
        })*
    };
}

make_perft_test! {
    perft_startpos(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        1,
        20,
        400,
        8902,
        197281,
        4865609,
        119060324
    );
    perft_kiwipete(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        1,
        48,
        2039,
        97862,
        4085603,
        193690690
    );
    perft_position_3(
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        1,
        14,
        191,
        2812,
        43238,
        674624,
        11030083,
        178633661
    );
    perft_position_4(
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        1,
        6,
        264,
        9467,
        422333,
        15833292
    );
    perft_position_5(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        1,
        44,
        1486,
        62379,
        2103487,
        89941194
    );
    perft_position_6(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        1,
        46,
        2079,
        89890,
        3894594,
        164075551
    );
    perft_960_position_333(
        "1rqbkrbn/1ppppp1p/1n6/p1N3p1/8/2P4P/PP1PPPP1/1RQBKRBN w FBfb - 0 9",
        1,
        29,
        502,
        14569,
        287739,
        8652810,
        191762235
    );
    perft_960_position_404(
        "rbbqn1kr/pp2p1pp/6n1/2pp1p2/2P4P/P7/BP1PPPP1/R1BQNNKR w HAha - 0 9",
        1,
        27,
        916,
        25798,
        890435,
        26302461,
        924181432
    );
    perft_960_position_789(
        "rqbbknr1/1ppp2pp/p5n1/4pp2/P7/1PP5/1Q1PPPPP/R1BBKNRN w GAga - 0 9",
        1,
        24,
        600,
        15347,
        408207,
        11029596,
        308553169
    );
    perft_960_position_726(
        "rkb2bnr/pp2pppp/2p1n3/3p4/q2P4/5NP1/PPP1PP1P/RKBNQBR1 w Aha - 0 9",
        1,
        29,
        861,
        24504,
        763454,
        22763215,
        731511256 
    );
}
