use crate::dot_chess::Error;
use crate::dot_chess::Result;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

#[derive(
    Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout,
)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl core::convert::Into<u8> for Piece {
    fn into(self) -> u8 {
        self as u8
    }
}

impl core::convert::TryFrom<u8> for Piece {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Piece::Pawn),
            1 => Ok(Piece::Knight),
            2 => Ok(Piece::Bishop),
            3 => Ok(Piece::Rook),
            4 => Ok(Piece::Queen),
            5 => Ok(Piece::King),
            _ => Err(Error::InvalidArgument),
        }
    }
}
