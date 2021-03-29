use crate::dot_chess::Result;
use super::{square::Square, Piece};
use core::convert::TryFrom;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

type PlyEncoded = u16;

#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Ply {
    from: Square,
    to: Square,
    promotion: Option<Piece>,
}

impl Ply {
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

    pub fn decode(encoded: PlyEncoded) -> Result<Self> {
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

    pub fn encode(&self) -> PlyEncoded {
        let promotion: u8 = match self.promotion {
            Some(piece) => piece.into(),
            None => 0,
        };

        let promotion = ((promotion & 0b00001111) as u16) << 12;
        let from = ((self.from.index() & 0b00111111) as u16) << 6;
        let to = (self.to.index() & 0b00111111) as u16;

        promotion | from | to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ply_encode() {
        let promotion = Some(Piece::Queen);
        let from: Square = 0b00110111u8.into();
        let to: Square = 0b00101001u8.into();

        let encoded = Ply::new(from, to, promotion).encode();

        assert_eq!(encoded, 0b01011101_11101001u16);
    }

    #[test]
    fn ply_decode() {
        let encoded = 0b01011101_11101001u16;

        let ply = Ply::decode(encoded).unwrap();

        assert_eq!(ply.promotion, Some(Piece::Queen));
        assert_eq!(ply.from.index(), 0b00110111u8);
        assert_eq!(ply.to.index(), 0b00101001u8);
    }
}
