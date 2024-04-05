use super::*;

#[test]
fn san_round_trip() {
    let board: Board = "3k2n1/7P/Q3p3/4BPp1/Q1Q4q/8/5B2/R3K2R w KQ g6 0 1".parse().unwrap();
    board.generate_moves(|mvs| {
        for mv in mvs {
            let san = format!("{}", display_san_move(&board, mv));
            let roundtripped_mv = parse_san_move(&board, &san).expect(&san);
            assert_eq!(roundtripped_mv, mv);
        }
        false
    });
}
