use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

use crate::dot_chess::Error;
use crate::dot_chess::Result;

#[derive(
    Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout,
)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
#[repr(u8)]
pub enum Side {
    White = 0,
    Black,
}

impl core::convert::Into<u8> for Side {
    fn into(self) -> u8 {
        self as u8
    }
}

impl core::convert::TryFrom<u8> for Side {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::White),
            1 => Ok(Self::Black),
            _ => Err(Error::InvalidArgument),
        }
    }
}

impl Side {
    pub fn flip(&self) -> Self {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}
