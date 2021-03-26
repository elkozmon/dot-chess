use crate::board::{Piece, Side, Square};
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

use super::{File, SquareIndex};

#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub enum Event {
    PieceLeftSquare(Side, Piece, SquareIndex),
    PieceEnteredSquare(Side, Piece, SquareIndex),
    NextTurn(Side),
    QueenCastlingRightLost(Side),
    KingCastlingRightLost(Side),
    EnPassantOpened(File),
    EnPassantClosed(File),
}
