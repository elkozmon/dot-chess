use crate::board::{Square, Piece, Side};

pub enum Event {
    PieceLeftSquare(Side, Piece, Square),
    PieceEnteredSquare(Side, Piece, Square),
    NextTurn(Side),
    QueenCastlingRightLost(Side),
    KingCastlingRightLost(Side),
    EnPassantOpened(Square),
    EnPassantClosed(Square),
}
