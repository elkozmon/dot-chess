use crate::common::{Error, Result};
use alloc::format;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

#[derive(
    Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout,
)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
#[repr(u8)]
pub enum Rank {
    _1 = 0,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
}

impl core::convert::Into<u8> for Rank {
    fn into(self) -> u8 {
        self as u8
    }
}

impl core::convert::TryFrom<u8> for Rank {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        use Rank::*;

        match value {
            n if n == _1 as u8 => Ok(_1),
            n if n == _2 as u8 => Ok(_2),
            n if n == _3 as u8 => Ok(_3),
            n if n == _4 as u8 => Ok(_4),
            n if n == _5 as u8 => Ok(_5),
            n if n == _6 as u8 => Ok(_6),
            n if n == _7 as u8 => Ok(_7),
            n if n == _8 as u8 => Ok(_8),
            n => Err(Error::InvalidArgument(format!("Invalid Rank index: {}", n))),
        }
    }
}

impl core::convert::Into<char> for Rank {
    fn into(self) -> char {
        use Rank::*;

        match self {
            _1 => '1',
            _2 => '2',
            _3 => '3',
            _4 => '4',
            _5 => '5',
            _6 => '6',
            _7 => '7',
            _8 => '8',
        }
    }
}

impl core::convert::TryFrom<char> for Rank {
    type Error = Error;

    fn try_from(value: char) -> Result<Self> {
        use Rank::*;

        match value.to_ascii_lowercase() {
            '1' => Ok(_1),
            '2' => Ok(_2),
            '3' => Ok(_3),
            '4' => Ok(_4),
            '5' => Ok(_5),
            '6' => Ok(_6),
            '7' => Ok(_7),
            '8' => Ok(_8),
            c => Err(Error::InvalidArgument(format!("Invalid Rank char: {}", c))),
        }
    }
}
