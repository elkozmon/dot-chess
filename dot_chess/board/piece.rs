use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

use crate::dot_chess::Error;

#[derive(
    Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout,
)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl core::convert::Into<u8> for Piece {
    fn into(self) -> u8 {
        match self {
            Piece::Pawn => 1,
            Piece::Knight => 2,
            Piece::Bishop => 3,
            Piece::Rook => 4,
            Piece::Queen => 5,
            Piece::King => 6,
        }
    }
}

impl core::convert::TryFrom<u8> for Piece {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Piece::Pawn),
            2 => Ok(Piece::Knight),
            3 => Ok(Piece::Bishop),
            4 => Ok(Piece::Rook),
            5 => Ok(Piece::Queen),
            6 => Ok(Piece::King),
            _ => Err(Error::InvalidArgument),
        }
    }
}
