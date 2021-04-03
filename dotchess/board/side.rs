use crate::common::{Error, Result};
use alloc::format;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

#[derive(
    Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout,
)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
#[repr(u8)]
pub enum Side {
    White = 0,
    Black,
}

const WHITE_STRING: &'static str = "White";
const BLACK_STRING: &'static str = "Black";

impl core::convert::Into<&'static str> for Side {
    fn into(self) -> &'static str {
        match self {
            Side::White => WHITE_STRING,
            Side::Black => BLACK_STRING,
        }
    }
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
            n => Err(Error::InvalidArgument(format!("Invalid Side index: {}", n))),
        }
    }
}

impl core::convert::Into<char> for Side {
    fn into(self) -> char {
        match self {
            Side::White => 'w',
            Side::Black => 'b',
        }
    }
}

impl core::convert::TryFrom<char> for Side {
    type Error = Error;

    fn try_from(value: char) -> Result<Self> {
        match value {
            'w' => Ok(Side::White),
            'b' => Ok(Side::Black),
            n => Err(Error::InvalidArgument(format!(
                "Invalid Side string: {}",
                n
            ))),
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
