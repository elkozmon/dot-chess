use crate::board::{Square, Piece, Player};

pub enum Event {
    PieceLeftSquare(Player, Piece, Square),
    PieceEnteredSquare(Player, Piece, Square),
    PlayersTurn(Player),
    QueenCastled(Player),
    KingCastled(Player),
    EnPassantOpened(Square),
    EnPassantClosed(Square),
}
