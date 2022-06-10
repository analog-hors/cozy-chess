crate::helpers::simple_enum! {
    /// A chess piece.
    /// Pieces are ordered by approximate material value.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
