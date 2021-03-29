use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

use crate::dot_chess::Error;
use crate::dot_chess::Result;

#[derive(
    Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout,
)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
pub enum Side {
    White,
    Black,
}

impl core::convert::Into<u8> for Side {
    fn into(self) -> u8 {
        match self {
            Side::White => Self::WHITE_VAL,
            Side::Black => Self::BLACK_VAL,
        }
    }
}

impl core::convert::TryFrom<u8> for Side {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            Self::WHITE_VAL => Ok(Self::White),
            Self::BLACK_VAL => Ok(Self::Black),
            _ => Err(Error::InvalidArgument),
        }
    }
}

impl Side {
    pub const VARIANTS: [Side; 2] = [Side::White, Side::Black];

    const WHITE_VAL: u8 = 0;
    const BLACK_VAL: u8 = 1;

    pub fn flip(&self) -> Self {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}
