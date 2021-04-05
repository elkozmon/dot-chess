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
pub enum Side {
    White = 0,
    Black,
}

impl core::convert::Into<u8> for Side {
    fn into(self) -> u8 {
        num::ToPrimitive::to_u8(&self).unwrap()
    }
}

impl core::convert::TryFrom<u8> for Side {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        num::FromPrimitive::from_u8(value)
            .ok_or_else(|| Error::InvalidArgument(format!("Invalid Side index: {}", value)))
    }
}

impl core::convert::Into<char> for Side {
    fn into(self) -> char {
        use Side::*;

        match self {
            White => 'w',
            Black => 'b',
        }
    }
}

impl core::convert::TryFrom<char> for Side {
    type Error = Error;

    fn try_from(value: char) -> Result<Self> {
        use Side::*;

        match value {
            'w' => Ok(White),
            'b' => Ok(Black),
            n => Err(Error::InvalidArgument(format!(
                "Invalid Side string: {}",
                n
            ))),
        }
    }
}

impl Side {
    pub fn flip(&self) -> Self {
        use Side::*;

        match self {
            White => Black,
            Black => White,
        }
    }

    pub fn as_str(&self) -> &'static str {
        use Side::*;

        match self {
            White => "white",
            Black => "black",
        }
    }
}
