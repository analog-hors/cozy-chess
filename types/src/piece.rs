crate::helpers::simple_enum! {
    /// A chess piece.
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum Piece {
        Pawn,
        Knight,
        Bishop,
        Rook,
        Queen,
        King
    }
}

crate::helpers::enum_char_conv! {
    Piece, PieceParseError {
        Pawn = 'p',
        Knight = 'n',
        Bishop = 'b',
        Rook = 'r',
        Queen = 'q',
        King = 'k'
    }
}
