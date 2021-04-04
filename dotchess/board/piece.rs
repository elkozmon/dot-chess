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
        use Piece::*;

        match value {
            n if n == Pawn as u8 => Ok(Pawn),
            n if n == Knight as u8 => Ok(Knight),
            n if n == Bishop as u8 => Ok(Bishop),
            n if n == Rook as u8 => Ok(Rook),
            n if n == Queen as u8 => Ok(Queen),
            n if n == King as u8 => Ok(King),
            n => Err(Error::InvalidArgument(format!(
                "Invalid Piece index: {}",
                n
            ))),
        }
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
