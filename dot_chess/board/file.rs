use crate::board::Error;
use crate::dot_chess::Result;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

#[derive(
    Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout,
)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
pub enum File {
    A = 0,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl core::convert::Into<u8> for File {
    fn into(self) -> u8 {
        self as u8
    }
}

impl core::convert::TryFrom<u8> for File {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            2 => Ok(Self::C),
            3 => Ok(Self::D),
            4 => Ok(Self::E),
            5 => Ok(Self::F),
            6 => Ok(Self::G),
            7 => Ok(Self::H),
            n => Err(Error::InvalidArgument(format!("Invalid File index: {}", n))),
        }
    }
}

impl core::convert::Into<char> for File {
    fn into(self) -> char {
        match self {
            File::A => 'a',
            File::B => 'b',
            File::C => 'c',
            File::D => 'd',
            File::E => 'e',
            File::F => 'f',
            File::G => 'g',
            File::H => 'h',
        }
    }
}

impl core::convert::TryFrom<char> for File {
    type Error = Error;

    fn try_from(value: char) -> Result<Self> {
        match value.to_ascii_lowercase() {
            'a' => Ok(Self::A),
            'b' => Ok(Self::B),
            'c' => Ok(Self::C),
            'd' => Ok(Self::D),
            'e' => Ok(Self::E),
            'f' => Ok(Self::F),
            'g' => Ok(Self::G),
            'h' => Ok(Self::H),
            c => Err(Error::InvalidArgument(format!("Invalid File char: {}", c))),
        }
    }
}
