use crate::common::{Error, Result};
use alloc::format;
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
        use File::*;

        match value {
            n if n == A as u8 => Ok(A),
            n if n == B as u8 => Ok(B),
            n if n == C as u8 => Ok(C),
            n if n == D as u8 => Ok(D),
            n if n == E as u8 => Ok(E),
            n if n == F as u8 => Ok(F),
            n if n == G as u8 => Ok(G),
            n if n == H as u8 => Ok(H),
            n => Err(Error::InvalidArgument(format!("Invalid File index: {}", n))),
        }
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
