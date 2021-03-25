use ink_storage::{
    collections::BinaryHeap,
    traits::{PackedLayout, SpreadLayout, StorageLayout},
};
use scale::{Decode, Encode};

#[derive(Copy, Clone, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(
    feature = "std",
    derive(PartialEq, Eq, scale_info::TypeInfo, StorageLayout)
)]
pub enum Direction {
    North = 0,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    pub fn positive(&self) -> bool {
        use Direction::*;

        match self {
            NorthWest | North | NorthEast | East => true,
            _ => false,
        }
    }

    pub fn negative(&self) -> bool {
        !self.positive()
    }
}
