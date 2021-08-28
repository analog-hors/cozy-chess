use super::*;

fn perft(board: &Board, depth: u8) -> u32 {
    if depth == 0 {
        1
    } else {
        let mut nodes = 0;
        board.generate_moves(&mut |moves| {
            for mv in moves {
                let mut board = board.clone();
                board.play_unchecked(mv);
                let child_nodes = perft(&board, depth - 1);
                nodes += child_nodes;
            }
            false
        });
        nodes
    }
}

macro_rules! make_perft_test {
    ($($name:ident($board:expr $(,$node:expr)*);)*) => {
        $(#[test]
        fn $name() {
            let board = $board.parse::<Board>().unwrap();
            const NODES: &'static [u32] = &[$($node),*];
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
}
