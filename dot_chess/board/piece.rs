use crate::common::{Error, Result};
use alloc::format;
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
            n => Err(Error::InvalidArgument(format!(
                "Invalid Piece index: {}",
                n
            ))),
        }
    }
}

impl core::convert::Into<char> for Piece {
    fn into(self) -> char {
        match self {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        }
    }
}

impl core::convert::TryFrom<char> for Piece {
    type Error = Error;

    fn try_from(value: char) -> Result<Self> {
        match value.to_ascii_lowercase() {
            'p' => Ok(Piece::Pawn),
            'r' => Ok(Piece::Rook),
            'n' => Ok(Piece::Knight),
            'b' => Ok(Piece::Bishop),
            'q' => Ok(Piece::Queen),
            'k' => Ok(Piece::King),
            c => Err(Error::InvalidArgument(format!("Invalid Piece char: {}", c))),
        }
    }
}
