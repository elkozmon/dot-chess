use crate::board::{PieceKind, Player, Rank, Square};

pub enum Event {
    FlipPieceOnSquare(Player, PieceKind, Square),
    FlipPlayerTurn,
    FlipQueenCastling(Player),
    FlipKingCastling(Player),
    FlipEnPassant(Player, Rank),
}
