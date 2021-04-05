use crate::common::Error;
use alloc::format;
use core::fmt::Write;
use num_derive::{FromPrimitive, ToPrimitive};
use scale::{Decode, Encode};

#[derive(Encode, Decode, Debug, Copy, Clone, ToPrimitive, FromPrimitive)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[repr(u8)]
pub enum GameOverReason {
    Checkmate = 0,
    Stalemate,
    InsufficientMatingMaterial,
    Resignation,
    ThreefoldRepetition,
    FiftyMoveRule,
    Abandonment,
    DrawAgreement,
}

impl core::convert::Into<u8> for GameOverReason {
    fn into(self) -> u8 {
        num::ToPrimitive::to_u8(&self).unwrap()
    }
}

impl core::convert::TryFrom<u8> for GameOverReason {
    type Error = Error;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        num::FromPrimitive::from_u8(value).ok_or_else(|| {
            Error::InvalidArgument(format!("Invalid GameOverReason index: {}", value))
        })
    }
}

impl GameOverReason {
    pub fn as_str(&self) -> &'static str {
        use GameOverReason::*;

        match self {
            Checkmate => "checkmate",
            Stalemate => "stalemate",
            InsufficientMatingMaterial => "insufficient mating material",
            Resignation => "resignation",
            ThreefoldRepetition => "threefold repetition",
            FiftyMoveRule => "fifty move rule",
            Abandonment => "abandonment",
            DrawAgreement => "draw agreement",
        }
    }
}
