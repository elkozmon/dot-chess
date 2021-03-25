use crate::board::{Piece, Side, Square};
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
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
    Checkmate(Side),
}
