use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

#[derive(Copy, Clone, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, scale_info::TypeInfo, StorageLayout)
)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
