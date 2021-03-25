use super::file::File;
use super::rank::Rank;
use crate::board::Error;
use core::ops::Range;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

pub type SquareIndex = u8;

pub const SQUARE_INDEX_RANGE: Range<u8> = 0..64;

#[derive(
    Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout,
)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
pub struct Square {
    index: SquareIndex,
    file: File,
    rank: Rank,
}

impl Square {
    pub fn new(file: File, rank: Rank) -> Self {
        let index = 8 * rank.index() + file.index();

        Self { index, file, rank }
    }

    pub fn from_index(index: SquareIndex) -> Self {
        let file = File::from_index(index & 7).unwrap();
        let rank = Rank::from_index(index >> 3).unwrap();

        Self { index, file, rank }
    }

    pub fn index(&self) -> SquareIndex {
        self.index
    }

    pub fn file(&self) -> File {
        self.file
    }

    pub fn rank(&self) -> Rank {
        self.rank
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
        let index = 9u8;
        let square = Square::from_index(index);

        assert_eq!(square.file(), File::B);
        assert_eq!(square.rank(), Rank::_2);
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
