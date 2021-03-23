use super::square::Square;
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
#[cfg_attr(
    feature = "std",
    derive(Clone, Debug, PartialEq, Eq, scale_info::TypeInfo, StorageLayout)
)]
pub struct Flags(u8);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ply {
    from: Square,
    to: Square,
    flags: Flags,
}

impl Ply {
    pub fn new(from: Square, to: Square, flags: Flags) -> Self {
        Self { from, to, flags }
    }

    pub fn decode(encoded: PlyEncoded) -> Self {
        let flags = ((encoded >> 12) & 0b00001111) as u8;
        let from = ((encoded >> 6) & 0b00111111) as u8;
        let to = (encoded & 0b00111111) as u8;

        Self {
            from: Square::from_index(from),
            to: Square::from_index(to),
            flags: Flags(flags),
        }
    }

    pub fn encode(&self) -> PlyEncoded {
        let flags = (self.flags.0 as u16 & 0b00001111) << 12;
        let from = (self.from.to_index() as u16 & 0b00111111) << 6;
        let to = self.to.to_index() as u16 & 0b00111111;

        flags | from | to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ply_encode() {
        let flags = 0b00001101u8;
        let from = 0b00110111u8;
        let to = 0b00101001u8;

        let encoded = Ply::new(
            Square::from_index(from),
            Square::from_index(to),
            Flags(flags),
        )
        .encode();

        assert_eq!(encoded, 0b11011101_11101001u16);
    }

    #[test]
    fn ply_decode() {
        let encoded = 0b11011101_11101001u16;

        let ply = Ply::decode(encoded);

        assert_eq!(ply.flags.0, 0b00001101u8);
        assert_eq!(ply.from.to_index(), 0b00110111u8);
        assert_eq!(ply.to.to_index(), 0b00101001u8);
    }
}
