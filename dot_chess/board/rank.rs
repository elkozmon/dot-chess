use crate::board::Error;
use crate::dot_chess::Result;
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

    fn try_from(value: u8) -> Result<Self> {
        match value {
            Self::RANK_1_VAL => Ok(Self::_1),
            Self::RANK_2_VAL => Ok(Self::_2),
            Self::RANK_3_VAL => Ok(Self::_3),
            Self::RANK_4_VAL => Ok(Self::_4),
            Self::RANK_5_VAL => Ok(Self::_5),
            Self::RANK_6_VAL => Ok(Self::_6),
            Self::RANK_7_VAL => Ok(Self::_7),
            Self::RANK_8_VAL => Ok(Self::_8),
            _ => Err(Error::InvalidArgument),
        }
    }
}

impl Rank {
    const RANK_1_VAL: u8 = 0;
    const RANK_2_VAL: u8 = 1;
    const RANK_3_VAL: u8 = 2;
    const RANK_4_VAL: u8 = 3;
    const RANK_5_VAL: u8 = 4;
    const RANK_6_VAL: u8 = 5;
    const RANK_7_VAL: u8 = 6;
    const RANK_8_VAL: u8 = 7;

    pub fn index(&self) -> u8 {
        match self {
            Self::_1 => Self::RANK_1_VAL,
            Self::_2 => Self::RANK_2_VAL,
            Self::_3 => Self::RANK_3_VAL,
            Self::_4 => Self::RANK_4_VAL,
            Self::_5 => Self::RANK_5_VAL,
            Self::_6 => Self::RANK_6_VAL,
            Self::_7 => Self::RANK_7_VAL,
            Self::_8 => Self::RANK_8_VAL,
        }
    }
}
