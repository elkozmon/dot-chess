use crate::board::Error;
use crate::dot_chess::Result;
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
        match value {
            0 => Ok(Self::_1),
            1 => Ok(Self::_2),
            2 => Ok(Self::_3),
            3 => Ok(Self::_4),
            4 => Ok(Self::_5),
            5 => Ok(Self::_6),
            6 => Ok(Self::_7),
            7 => Ok(Self::_8),
            n => Err(Error::InvalidArgument(format!("Invalid Rank index: {}", n))),
        }
    }
}

impl core::convert::Into<char> for Rank {
    fn into(self) -> char {
        match self {
            Rank::_1 => '1',
            Rank::_2 => '2',
            Rank::_3 => '3',
            Rank::_4 => '4',
            Rank::_5 => '5',
            Rank::_6 => '6',
            Rank::_7 => '7',
            Rank::_8 => '8',
        }
    }
}

impl core::convert::TryFrom<char> for Rank {
    type Error = Error;

    fn try_from(value: char) -> Result<Self> {
        match value.to_ascii_lowercase() {
            '1' => Ok(Self::_1),
            '2' => Ok(Self::_2),
            '3' => Ok(Self::_3),
            '4' => Ok(Self::_4),
            '5' => Ok(Self::_5),
            '6' => Ok(Self::_6),
            '7' => Ok(Self::_7),
            '8' => Ok(Self::_8),
            c => Err(Error::InvalidArgument(format!("Invalid Rank char: {}", c))),
        }
    }
}
