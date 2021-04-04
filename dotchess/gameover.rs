use crate::common::Error;
use alloc::format;
use core::fmt::Write;
use scale::{Decode, Encode};
use GameOverReason::*;

#[derive(Encode, Decode, Debug, Copy, Clone)]
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
        self as u8
    }
}

impl core::convert::TryFrom<u8> for GameOverReason {
    type Error = Error;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        match value {
            n if n == Checkmate as u8 => Ok(Checkmate),
            n if n == Stalemate as u8 => Ok(Stalemate),
            n if n == InsufficientMatingMaterial as u8 => Ok(InsufficientMatingMaterial),
            n if n == Resignation as u8 => Ok(Resignation),
            n if n == ThreefoldRepetition as u8 => Ok(ThreefoldRepetition),
            n if n == FiftyMoveRule as u8 => Ok(FiftyMoveRule),
            n if n == Abandonment as u8 => Ok(Abandonment),
            n if n == DrawAgreement as u8 => Ok(DrawAgreement),
            n => Err(Error::InvalidArgument(format!(
                "Invalid GameOverReason index: {}",
                n
            ))),
        }
    }
}

impl GameOverReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameOverReason::Checkmate => "checkmate",
            GameOverReason::Stalemate => "stalemate",
            GameOverReason::InsufficientMatingMaterial => "insufficient mating material",
            GameOverReason::Resignation => "resignation",
            GameOverReason::ThreefoldRepetition => "threefold repetition",
            GameOverReason::FiftyMoveRule => "fifty move rule",
            GameOverReason::Abandonment => "abandonment",
            GameOverReason::DrawAgreement => "draw agreement",
        }
    }
}
