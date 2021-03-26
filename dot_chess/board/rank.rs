use crate::board::Error;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

#[derive(
    Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout,
)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
pub enum Rank {
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
}

impl core::convert::TryFrom<u8> for Rank {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::_1),
            1 => Ok(Self::_2),
            2 => Ok(Self::_3),
            3 => Ok(Self::_4),
            4 => Ok(Self::_5),
            5 => Ok(Self::_6),
            6 => Ok(Self::_7),
            7 => Ok(Self::_8),
            _ => Err(Error::InvalidArgument),
        }
    }
}

impl Rank {
    pub fn index(&self) -> u8 {
        match self {
            Self::_1 => 0,
            Self::_2 => 1,
            Self::_3 => 2,
            Self::_4 => 3,
            Self::_5 => 4,
            Self::_6 => 5,
            Self::_7 => 6,
            Self::_8 => 7,
        }
    }
}
