use super::file::File;
use super::rank::Rank;
use core::convert::TryFrom;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

#[derive(
    Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout,
)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
pub struct Square(u8);

impl core::convert::From<u8> for Square {
    fn from(val: u8) -> Self {
        Self(val)
    }
}

impl core::convert::Into<File> for Square {
    fn into(self) -> File {
        File::try_from(self.index() & 7).unwrap()
    }
}

impl core::convert::Into<Rank> for Square {
    fn into(self) -> Rank {
        Rank::try_from(self.index() >> 3).unwrap()
    }
}

impl Square {
    pub const A1: Self = Self(0);
    pub const C1: Self = Self(2);
    pub const D1: Self = Self(3);
    pub const E1: Self = Self(4);
    pub const F1: Self = Self(5);
    pub const G1: Self = Self(6);
    pub const H1: Self = Self(7);
    pub const A8: Self = Self(56);
    pub const C8: Self = Self(58);
    pub const D8: Self = Self(59);
    pub const E8: Self = Self(60);
    pub const F8: Self = Self(61);
    pub const G8: Self = Self(62);
    pub const H8: Self = Self(63);

    pub fn new(file: File, rank: Rank) -> Self {
        Self::from(8 * rank.index() + file.index())
    }

    pub fn index(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_to_index() {
        let square = Square::new(File::B, Rank::_2);

        assert_eq!(square.index(), 9u8);
    }

    #[test]
    fn square_from_index() {
        let square: Square = 9u8.into();
        let file: File = square.into();
        let rank: Rank = square.into();

        assert_eq!(file, File::B);
        assert_eq!(rank, Rank::_2);
    }

    #[test]
    fn square_h8_index() {
        let square = Square::new(File::H, Rank::_8);

        assert_eq!(square.index(), 63);
    }

    #[test]
    fn square_a1_index() {
        let square = Square::new(File::A, Rank::_1);

        assert_eq!(square.index(), 0);
    }

    #[test]
    fn square_a2_index() {
        let square = Square::new(File::A, Rank::_2);

        assert_eq!(square.index(), 8);
    }

    #[test]
    fn square_b2_index() {
        let square = Square::new(File::B, Rank::_2);

        assert_eq!(square.index(), 9);
    }
}
