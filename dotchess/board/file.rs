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
        num::ToPrimitive::to_u8(&self).unwrap()
    }
}

impl core::convert::TryFrom<u8> for File {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        num::FromPrimitive::from_u8(value)
            .ok_or_else(|| Error::InvalidArgument(format!("Invalid File index: {}", value)))
    }
}

impl core::convert::Into<char> for File {
    fn into(self) -> char {
        use File::*;

        match self {
            A => 'a',
            B => 'b',
            C => 'c',
            D => 'd',
            E => 'e',
            F => 'f',
            G => 'g',
            H => 'h',
        }
    }
}

impl core::convert::TryFrom<char> for File {
    type Error = Error;

    fn try_from(value: char) -> Result<Self> {
        use File::*;

        match value.to_ascii_lowercase() {
            'a' => Ok(A),
            'b' => Ok(B),
            'c' => Ok(C),
            'd' => Ok(D),
            'e' => Ok(E),
            'f' => Ok(F),
            'g' => Ok(G),
            'h' => Ok(H),
            c => Err(Error::InvalidArgument(format!("Invalid File char: {}", c))),
        }
    }
}
