use crate::board::Error;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

#[derive(
    Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout,
)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl core::convert::TryFrom<u8> for File {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            2 => Ok(Self::C),
            3 => Ok(Self::D),
            4 => Ok(Self::E),
            5 => Ok(Self::F),
            6 => Ok(Self::G),
            7 => Ok(Self::H),
            _ => Err(Error::InvalidArgument),
        }
    }
}

impl File {
    pub const VARIANTS: [File; 8] = [
        File::A,
        File::B,
        File::C,
        File::D,
        File::E,
        File::F,
        File::G,
        File::H,
    ];

    pub fn index(&self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
            Self::E => 4,
            Self::F => 5,
            Self::G => 6,
            Self::H => 7,
        }
    }
}
