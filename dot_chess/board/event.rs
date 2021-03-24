use crate::board::{Square, Piece, Side};
use scale::{Decode, Encode};
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};

#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(
    feature = "std",
    derive(Clone, Debug, PartialEq, Eq, scale_info::TypeInfo, StorageLayout)
)]
pub enum Event {
    PieceLeftSquare(Side, Piece, Square),
    PieceEnteredSquare(Side, Piece, Square),
    NextTurn(Side),
    QueenCastlingRightLost(Side),
    KingCastlingRightLost(Side),
    EnPassantOpened(Square),
    EnPassantClosed(Square),
    Stalemate,
    InsufficientMatingMaterial,
    Check(Side),
    Checkmate(Side)
}
