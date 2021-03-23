use crate::board::Error;

pub type SquareIndex = u8;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl File {
    pub const VARIANTS: [File; 8] = [
        File::A,
        File::B,
        File::C,
        File::D,
        File::E,
        File::F,
        File::G,
        File::H,
    ];

    pub fn to_index(&self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
            Self::E => 4,
            Self::F => 5,
            Self::G => 6,
            Self::H => 7,
        }
    }

    pub fn from_index(index: u8) -> Result<Self, Error> {
        match index {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            2 => Ok(Self::C),
            3 => Ok(Self::D),
            4 => Ok(Self::E),
            5 => Ok(Self::F),
            6 => Ok(Self::G),
            7 => Ok(Self::H),
            _ => Err(Error::InvalidArgument),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Rank {
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
}

impl Rank {
    pub fn to_index(&self) -> u8 {
        match self {
            Self::_1 => 0,
            Self::_2 => 1,
            Self::_3 => 2,
            Self::_4 => 3,
            Self::_5 => 4,
            Self::_6 => 5,
            Self::_7 => 6,
            Self::_8 => 7,
        }
    }

    pub fn from_index(index: u8) -> Result<Self, Error> {
        match index {
            0 => Ok(Self::_1),
            1 => Ok(Self::_2),
            2 => Ok(Self::_3),
            3 => Ok(Self::_4),
            4 => Ok(Self::_5),
            5 => Ok(Self::_6),
            6 => Ok(Self::_7),
            7 => Ok(Self::_8),
            _ => Err(Error::InvalidArgument),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Square {
    file: File,
    rank: Rank,
}

impl Square {
    pub fn new(file: File, rank: Rank) -> Self {
        Self { file, rank }
    }

    pub fn from_index(index: SquareIndex) -> Self {
        let file = File::from_index(index & 7).unwrap();
        let rank = Rank::from_index(index >> 3).unwrap();

        Self { file, rank }
    }

    pub fn to_index(&self) -> SquareIndex {
        8 * self.rank.to_index() + self.file.to_index()
    }

    pub fn file(&self) -> &File {
        &self.file
    }

    pub fn rank(&self) -> &Rank {
        &self.rank
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_to_index() {
        let square = Square::new(File::B, Rank::_2);

        assert_eq!(square.to_index(), 9u8);
    }

    #[test]
    fn square_from_index() {
        let index = 9u8;
        let square = Square::from_index(index);

        
        assert_eq!(*square.file(), File::B);
        assert_eq!(*square.rank(), Rank::_2);
    }

    #[test]
    fn square_h8_index() {
        let square = Square::new(File::H, Rank::_8);

        assert_eq!(square.to_index(), 63);
    }

    #[test]
    fn square_a1_index() {
        let square = Square::new(File::A, Rank::_1);

        assert_eq!(square.to_index(), 0);
    }

    #[test]
    fn square_a2_index() {
        let square = Square::new(File::A, Rank::_2);

        assert_eq!(square.to_index(), 8);
    }

    #[test]
    fn square_b2_index() {
        let square = Square::new(File::B, Rank::_2);

        assert_eq!(square.to_index(), 9);
    }
}
