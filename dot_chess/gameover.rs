use crate::dot_chess::{Error, Result};
use alloc::format;
use core::fmt::Write;
use scale::{Decode, Encode};

#[derive(Encode, Decode, Debug, Copy, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[repr(u8)]
pub enum GameOverReason {
    Checkmate = 0,
    Stalemate,
    InsufficientMatingMaterial,
    Resignation,
    Repetition,
    FiftyMoveRule,
}

impl core::convert::Into<u8> for GameOverReason {
    fn into(self) -> u8 {
        self as u8
    }
}

impl core::convert::TryFrom<u8> for GameOverReason {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Checkmate),
            1 => Ok(Self::Stalemate),
            2 => Ok(Self::InsufficientMatingMaterial),
            3 => Ok(Self::Resignation),
            4 => Ok(Self::Repetition),
            5 => Ok(Self::FiftyMoveRule),
            n => Err(Error::InvalidArgument(format!(
                "Invalid GameOverReason index: {}",
                n
            ))),
        }
    }
}
