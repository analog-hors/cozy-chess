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

#[test]
fn handles_canonical_san() {
    let board: Board = "3k2n1/7P/Q3p3/4BPp1/Q1Q4q/8/5B2/R3K2R w KQ g6 0 1".parse().unwrap();
    let moves = [
        ("a6c8", "Qac8+"), ("a6a8", "Qa8+"), ("a6b7", "Qb7"), ("a6a7", "Qa7"),
        ("a6e6", "Qaxe6"), ("a6d6", "Qd6#"), ("a6c6", "Q6c6"), ("a6b6", "Qb6+"),
        ("a6b5", "Q6b5"), ("a6a5", "Q6a5+"), ("e5h8", "Bh8"), ("e5b8", "Bb8"),
        ("e5g7", "Bg7"), ("e5c7", "Bc7+"), ("e5f6", "Bf6+"), ("e5d6", "Bd6"),
        ("e5f4", "Bf4"), ("e5d4", "Bd4"), ("e5g3", "Beg3"), ("e5c3", "Bc3"),
        ("e5h2", "Bh2"), ("e5b2", "Bb2"), ("c4c8", "Qcc8+"), ("c4c7", "Qc7#"),
        ("c4e6", "Qcxe6"), ("c4c6", "Qcc6"), ("c4d5", "Qd5+"), ("c4c5", "Qc5"),
        ("c4b5", "Qcb5"), ("c4h4", "Qxh4"), ("c4g4", "Qg4"), ("c4f4", "Qf4"),
        ("c4e4", "Qe4"), ("c4d4", "Qd4+"), ("c4b4", "Qcb4"), ("c4d3", "Qd3+"),
        ("c4c3", "Qc3"), ("c4b3", "Qcb3"), ("c4e2", "Qe2"), ("c4c2", "Qcc2"),
        ("c4a2", "Qca2"), ("c4f1", "Qf1"), ("c4c1", "Qc1"), ("a4e8", "Qe8+"),
        ("a4d7", "Qd7+"), ("a4c6", "Qa4c6"), ("a4b5", "Qa4b5"), ("a4a5", "Q4a5+"),
        ("a4b4", "Qab4"), ("a4b3", "Qab3"), ("a4a3", "Qa3"), ("a4c2", "Qac2"),
        ("a4a2", "Qaa2"), ("a4d1", "Qd1+"), ("f2h4", "Bxh4"), ("f2g3", "Bfg3"),
        ("h1h4", "Rxh4"), ("h1h3", "Rh3"), ("h1h2", "Rh2"), ("h1g1", "Rg1"),
        ("h1f1", "Rf1"), ("e1e2", "Ke2"), ("e1d2", "Kd2"), ("e1f1", "Kf1"),
        ("e1d1", "Kd1"), ("a1a3", "Ra3"), ("a1a2", "Ra2"), ("a1d1", "Rd1+"),
        ("a1c1", "Rc1"), ("a1b1", "Rb1"), ("e1h1", "O-O"), ("e1a1", "O-O-O+"),
        ("h7g8q", "hxg8=Q+"), ("h7g8r", "hxg8=R+"), ("h7g8b", "hxg8=B"), ("h7g8n", "hxg8=N"),
        ("f5e6", "fxe6"), ("h7h8q", "h8=Q"), ("h7h8r", "h8=R"), ("h7h8b", "h8=B"),
        ("h7h8n", "h8=N"), ("f5f6", "f6"), ("f5g6", "fxg6"),
    ];
    
    for (mv, san) in moves {
        let mv = mv.parse().unwrap();
        assert_eq!(san, format!("{}", display_san_move(&board, mv)));
        assert_eq!(mv, parse_san_move(&board, san).expect(&san));
    }
}
