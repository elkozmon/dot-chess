use super::square::Square;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

type PlyEncoded = u16;

/// Promotion codes
///
/// 0  no promotion       (000)
/// 1  knight promotion   (001)
/// 2  bishop promotion   (010)
/// 3  rook promotion     (011)
/// 4  queen promotion    (100)
#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Promotion(u8);

impl Promotion {
    pub const NO_PROMOTION: Self = Self(0);
    pub const KNIGHT_PROMOTION: Self = Self(1);
    pub const BISHOP_PROMOTION: Self = Self(2);
    pub const ROOK_PROMOTION: Self = Self(3);
    pub const QUEEN_PROMOTION: Self = Self(4);
}

#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Ply {
    from: Square,
    to: Square,
    promotion: Promotion,
}

impl Ply {
    pub fn new(from: Square, to: Square, promotion: Promotion) -> Self {
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

    pub fn decode(encoded: PlyEncoded) -> Self {
        let flags = ((encoded >> 12) & 0b00001111) as u8;
        let flags = Promotion(flags);

        let from = ((encoded >> 6) & 0b00111111) as u8;
        let from: Square = from.into();

        let to = (encoded & 0b00111111) as u8;
        let to: Square = to.into();

        Self {
            from,
            to,
            promotion: flags,
        }
    }

    pub fn encode(&self) -> PlyEncoded {
        let flags = ((self.promotion.0 & 0b00001111) as u16) << 12;
        let from = ((self.from.index() & 0b00111111) as u16) << 6;
        let to = (self.to.index() & 0b00111111) as u16;

        flags | from | to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ply_encode() {
        let flags = Promotion(0b00000101u8);
        let from: Square = 0b00110111u8.into();
        let to: Square = 0b00101001u8.into();

        let encoded = Ply::new(from, to, flags).encode();

        assert_eq!(encoded, 0b01011101_11101001u16);
    }

    #[test]
    fn ply_decode() {
        let encoded = 0b01011101_11101001u16;

        let ply = Ply::decode(encoded);

        assert_eq!(ply.promotion.0, 0b00000101u8);
        assert_eq!(ply.from.index(), 0b00110111u8);
        assert_eq!(ply.to.index(), 0b00101001u8);
    }
}
