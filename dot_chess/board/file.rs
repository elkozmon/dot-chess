use crate::board::Error;
use crate::dot_chess::Result;
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

    fn try_from(value: u8) -> Result<Self> {
        match value {
            Self::FILE_A_VAL => Ok(Self::A),
            Self::FILE_B_VAL => Ok(Self::B),
            Self::FILE_C_VAL => Ok(Self::C),
            Self::FILE_D_VAL => Ok(Self::D),
            Self::FILE_E_VAL => Ok(Self::E),
            Self::FILE_F_VAL => Ok(Self::F),
            Self::FILE_G_VAL => Ok(Self::G),
            Self::FILE_H_VAL => Ok(Self::H),
            _ => Err(Error::InvalidArgument),
        }
    }
}

impl File {
    const FILE_A_VAL: u8 = 0;
    const FILE_B_VAL: u8 = 1;
    const FILE_C_VAL: u8 = 2;
    const FILE_D_VAL: u8 = 3;
    const FILE_E_VAL: u8 = 4;
    const FILE_F_VAL: u8 = 5;
    const FILE_G_VAL: u8 = 6;
    const FILE_H_VAL: u8 = 7;

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
            Self::A => Self::FILE_A_VAL,
            Self::B => Self::FILE_B_VAL,
            Self::C => Self::FILE_C_VAL,
            Self::D => Self::FILE_D_VAL,
            Self::E => Self::FILE_E_VAL,
            Self::F => Self::FILE_F_VAL,
            Self::G => Self::FILE_G_VAL,
            Self::H => Self::FILE_H_VAL,
        }
    }
}
