use crate::board::{PieceKind, Player, Rank, Square};

pub enum Event {
    PieceLeftSquare(Player, PieceKind, Square),
    PieceEnteredSquare(Player, PieceKind, Square),
    PlayersTurn(Player),
    QueenCastled(Player),
    KingCastled(Player),
    EnPassantOpened(Square),
    EnPassantClosed(Square),
}
