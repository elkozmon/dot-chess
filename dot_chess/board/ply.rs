use super::{square::Square, SquareIndex};
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

type PlyEncoded = u16;

/// Ply flags codes
///
/// 0   0000  quiet move
/// 1   0001  double pawn push
/// 2   0010  king castle
/// 3   0011  queen castle
/// 4   0100  capture
/// 5   0101  ep-capture
/// 8   1000  knight-promotion
/// 9   1001  bishop-promotion
/// 10  1010  rook-promotion
/// 11  1011  queen-promotion
/// 12  1100  knight-promo capture
/// 13  1101  bishop-promo capture
/// 14  1110  rook-promo capture
/// 15  1111  queen-promo capture
#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Flags(u8);

#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Ply {
    from: Square,
    to: Square,
    flags: Flags,
}

impl Ply {
    pub fn new(from: Square, to: Square, flags: Flags) -> Self {
        Self { from, to, flags }
    }

    pub fn from(&self) -> Square {
        self.from
    }

    pub fn to(&self) -> Square {
        self.to
    }

    pub fn decode(encoded: PlyEncoded) -> Self {
        let flags = ((encoded >> 12) & 0b00001111) as u8;
        let flags = Flags(flags);

        let from = ((encoded >> 6) & 0b00111111) as u8;
        let from = Square::from_index(from);

        let to = (encoded & 0b00111111) as u8;
        let to = Square::from_index(to);

        Self { from, to, flags }
    }

    pub fn encode(&self) -> PlyEncoded {
        let flags = ((self.flags.0 & 0b00001111) as u16) << 12;
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
        let flags = Flags(0b00001101u8);
        let from = Square::from_index(0b00110111u8);
        let to = Square::from_index(0b00101001u8);

        let encoded = Ply::new(from, to, flags).encode();

        assert_eq!(encoded, 0b11011101_11101001u16);
    }

    #[test]
    fn ply_decode() {
        let encoded = 0b11011101_11101001u16;

        let ply = Ply::decode(encoded);

        assert_eq!(ply.flags.0, 0b00001101u8);
        assert_eq!(ply.from.index(), 0b00110111u8);
        assert_eq!(ply.to.index(), 0b00101001u8);
    }
}
