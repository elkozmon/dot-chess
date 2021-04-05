use crate::common::{Error, Result};
use alloc::format;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use num_derive::{FromPrimitive, ToPrimitive};
use scale::{Decode, Encode};

#[derive(
    Copy,
    Clone,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Encode,
    Decode,
    SpreadLayout,
    PackedLayout,
    ToPrimitive,
    FromPrimitive,
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
        num::ToPrimitive::to_u8(&self).unwrap()
    }
}

impl core::convert::TryFrom<u8> for Piece {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        num::FromPrimitive::from_u8(value)
            .ok_or_else(|| Error::InvalidArgument(format!("Invalid Piece index: {}", value)))
    }
}

impl core::convert::Into<char> for Piece {
    fn into(self) -> char {
        use Piece::*;

        match self {
            Pawn => 'p',
            Knight => 'n',
            Bishop => 'b',
            Rook => 'r',
            Queen => 'q',
            King => 'k',
        }
    }
}

impl core::convert::TryFrom<char> for Piece {
    type Error = Error;

    fn try_from(value: char) -> Result<Self> {
        use Piece::*;

        match value.to_ascii_lowercase() {
            'p' => Ok(Pawn),
            'r' => Ok(Rook),
            'n' => Ok(Knight),
            'b' => Ok(Bishop),
            'q' => Ok(Queen),
            'k' => Ok(King),
            c => Err(Error::InvalidArgument(format!("Invalid Piece char: {}", c))),
        }
    }
}
