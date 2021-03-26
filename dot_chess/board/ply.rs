use super::{square::Square, Piece, Side, SquareIndex};
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

type PlyEncoded = u16;

// TODO refactor
/// Ply flags codes
///
/// 0   0000  knight-promotion
/// 1   0001  bishop-promotion
/// 2   0010  rook-promotion
/// 3   0011  queen-promotion
#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Flags(u8);

impl Flags {
    pub const DEFAULT: Self = Self(0);
    pub const KNIGHT_PROMOTION: Self = Self(0);
    pub const BISHOP_PROMOTION: Self = Self(1);
    pub const ROOK_PROMOTION: Self = Self(2);
    pub const QUEEN_PROMOTION: Self = Self(3);
}

#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct Ply {
    from: SquareIndex,
    to: SquareIndex,
    flags: Flags,
}

impl Ply {
    pub fn new(from: SquareIndex, to: SquareIndex, flags: Flags) -> Self {
        Self { from, to, flags }
    }

    pub fn from(&self) -> SquareIndex {
        self.from
    }

    pub fn to(&self) -> SquareIndex {
        self.to
    }

    pub fn decode(encoded: PlyEncoded) -> Self {
        let flags = ((encoded >> 12) & 0b00000011) as u8;
        let flags = Flags(flags);

        let from = ((encoded >> 6) & 0b00111111) as u8;

        let to = (encoded & 0b00111111) as u8;

        Self { from, to, flags }
    }

    pub fn encode(&self) -> PlyEncoded {
        let flags = ((self.flags.0 & 0b00000011) as u16) << 12;
        let from = ((self.from & 0b00111111) as u16) << 6;
        let to = (self.to & 0b00111111) as u16;

        flags | from | to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ply_encode() {
        let flags = Flags(0b00000001u8);
        let from = 0b00110111u8;
        let to = 0b00101001u8;

        let encoded = Ply::new(from, to, flags).encode();

        assert_eq!(encoded, 0b00011101_11101001u16);
    }

    #[test]
    fn ply_decode() {
        let encoded = 0b00011101_11101001u16;

        let ply = Ply::decode(encoded);

        assert_eq!(ply.flags.0, 0b00000001u8);
        assert_eq!(ply.from, 0b00110111u8);
        assert_eq!(ply.to, 0b00101001u8);
    }
}
