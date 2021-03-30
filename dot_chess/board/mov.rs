use super::{square::Square, File, Piece, Rank};
use crate::dot_chess::{Error, Result};
use core::convert::TryFrom;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};
use std::convert::TryInto;

type MovEncoded = u16;

#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Mov {
    from: Square,
    to: Square,
    promotion: Option<Piece>,
}

impl core::convert::TryFrom<&str> for Mov {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        let error_invalid_mov = || Error::InvalidArgument(format!("Invalid move: {}", value));

        let mut chars = value.chars();

        let file: File = chars.next().ok_or_else(error_invalid_mov)?.try_into()?;
        let rank: Rank = chars.next().ok_or_else(error_invalid_mov)?.try_into()?;
        let from = Square::new(file, rank);

        let file: File = chars.next().ok_or_else(error_invalid_mov)?.try_into()?;
        let rank: Rank = chars.next().ok_or_else(error_invalid_mov)?.try_into()?;
        let to = Square::new(file, rank);

        let promotion: Option<Piece> = match chars.next() {
            Some(char) => Some(char.try_into()?),
            None => None,
        };

        Ok(Self {
            from,
            to,
            promotion,
        })
    }
}

impl Mov {
    pub fn new(from: Square, to: Square, promotion: Option<Piece>) -> Self {
        Self {
            from,
            to,
            promotion,
        }
    }

    pub fn from(&self) -> Square {
        self.from
    }

    pub fn to(&self) -> Square {
        self.to
    }

    pub fn promotion(&self) -> Option<Piece> {
        self.promotion
    }

    pub fn decode(encoded: MovEncoded) -> Result<Self> {
        let promotion = match ((encoded >> 12) & 0b00001111) as u8 {
            0 => None,
            n => Some(<Piece as TryFrom<u8>>::try_from(n)?),
        };

        let from = ((encoded >> 6) & 0b00111111) as u8;
        let from: Square = from.into();

        let to = (encoded & 0b00111111) as u8;
        let to: Square = to.into();

        Ok(Self {
            from,
            to,
            promotion,
        })
    }

    pub fn encode(&self) -> MovEncoded {
        let promotion: u8 = match self.promotion {
            Some(piece) => piece.into(),
            None => 0,
        };

        let promotion = ((promotion & 0b00001111) as u16) << 12;

        let from: u8 = self.from.into();
        let from = ((from & 0b00111111) as u16) << 6;

        let to: u8 = self.to.into();
        let to = (to & 0b00111111) as u16;

        promotion | from | to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mov_encode() {
        let promotion = Some(Piece::Queen);
        let from: Square = 0b00110111u8.into();
        let to: Square = 0b00101001u8.into();

        let encoded = Mov::new(from, to, promotion).encode();

        assert_eq!(encoded, 0b01001101_11101001u16);
    }

    #[test]
    fn mov_decode() {
        let encoded = 0b01001101_11101001u16;

        let mov = Mov::decode(encoded).unwrap();
        let from: u8 = mov.from.into();
        let to: u8 = mov.to.into();

        assert_eq!(mov.promotion, Some(Piece::Queen));
        assert_eq!(from, 0b00110111u8);
        assert_eq!(to, 0b00101001u8);
    }
}
