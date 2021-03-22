use crate::dot_chess::Error;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

pub type SquareIndex = u8;
pub type MoveFlags = u8;
pub type MoveEncoded = u16;
type BitBoard = u64;

const UNIVERSAL_SET: BitBoard = 0xffffffffffffffff;
const EMPTY_SET: BitBoard = 0;

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
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
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

    pub fn to_bitboard(&self) -> BitBoard {
        1 << self.to_index()
    }

    pub fn file(&self) -> File {
        self.file
    }

    pub fn rank(&self) -> Rank {
        self.rank
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Move {
    from: Square,
    to: Square,
    flags: MoveFlags,
}

impl Move {
    pub fn new(from: Square, to: Square, flags: MoveFlags) -> Self {
        Self { from, to, flags }
    }

    pub fn decode(encoded: MoveEncoded) -> Self {
        let flags = ((encoded >> 12) & 0b00001111) as u8;
        let from = ((encoded >> 6) & 0b00111111) as u8;
        let to = (encoded & 0b00111111) as u8;

        Self {
            from: Square::from_index(from),
            to: Square::from_index(to),
            flags,
        }
    }

    pub fn encode(&self) -> MoveEncoded {
        let flags = (self.flags as u16 & 0b00001111) << 12;
        let from = (self.from.to_index() as u16 & 0b00111111) << 6;
        let to = self.to.to_index() as u16 & 0b00111111;

        flags | from | to
    }
}

#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, scale_info::TypeInfo, StorageLayout)
)]
pub struct Board {
    black: BitBoard,
    white: BitBoard,
    kings: BitBoard,
    queens: BitBoard,
    rooks: BitBoard,
    bishops: BitBoard,
    knights: BitBoard,
    pawns: BitBoard,
}

impl Board {
    pub fn default() -> Self {
        Self {
            black: 0xffff000000000000,
            white: 0xffff,
            kings: 0x1000000000000010,
            queens: 0x800000000000008,
            rooks: 0x8100000000000081,
            bishops: 0x2400000000000024,
            knights: 0x4200000000000042,
            pawns: 0xff00000000ff00,
        }
    }

    /// Returns array of 64 8-bit integers representing current state of the board
    /// in following square order: A1, A2, ..., B1, B2, ..., H8
    ///
    /// 0 - Empty square
    /// 1 - Pawn
    /// 2 - Knight
    /// 3 - Bishop
    /// 4 - Rook
    /// 5 - Queen
    /// 6 - King
    ///
    /// Negative integers represent black pieces
    /// Positive integers represent white pieces
    pub fn to_array(&self) -> [i8; 64] {
        let mut board = [0; 64];

        for i in 0..64 {
            let mut square = 0;

            let is_black = ((self.black >> i) & 1) == 1;
            let is_white = ((self.white >> i) & 1) == 1;

            if !(is_white || is_black) {
                // Square is empty
                continue;
            }

            if ((self.pawns >> i) & 1) == 1 {
                square = 1;
            } else if ((self.knights >> i) & 1) == 1 {
                square = 2;
            } else if ((self.bishops >> i) & 1) == 1 {
                square = 3;
            } else if ((self.rooks >> i) & 1) == 1 {
                square = 4;
            } else if ((self.queens >> i) & 1) == 1 {
                square = 5;
            } else if ((self.kings >> i) & 1) == 1 {
                square = 6;
            }

            if is_black {
                board[i] = -square;
            } else {
                board[i] = square;
            }
        }

        board
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

        assert_eq!(square.file(), File::B);
        assert_eq!(square.rank(), Rank::_2);
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

    #[test]
    fn square_h8_bitboard() {
        let square = Square::new(File::H, Rank::_8);

        assert_eq!(
            square.to_bitboard(),
            0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000u64
        );
    }

    #[test]
    fn square_a1_bitboard() {
        let square = Square::new(File::A, Rank::_1);

        assert_eq!(
            square.to_bitboard(),
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001u64
        );
    }

    #[test]
    fn square_a2_bitboard() {
        let square = Square::new(File::A, Rank::_2);

        assert_eq!(
            square.to_bitboard(),
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000001_00000000u64
        );
    }

    #[test]
    fn square_b2_bitboard() {
        let square = Square::new(File::B, Rank::_2);

        assert_eq!(
            square.to_bitboard(),
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000010_00000000u64
        );
    }

    #[test]
    fn board_pawn_positions() {
        let board = Board::default();

        assert_eq!(
            board.pawns,
            0b00000000_11111111_00000000_00000000_00000000_00000000_11111111_00000000u64
        );
    }

    #[test]
    fn board_rook_positions() {
        let board = Board::default();

        assert_eq!(
            board.rooks,
            0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_10000001u64
        );
    }

    #[test]
    fn board_knight_positions() {
        let board = Board::default();

        assert_eq!(
            board.knights,
            0b01000010_00000000_00000000_00000000_00000000_00000000_00000000_01000010u64
        );
    }

    #[test]
    fn board_bishop_positions() {
        let board = Board::default();

        assert_eq!(
            board.bishops,
            0b00100100_00000000_00000000_00000000_00000000_00000000_00000000_00100100u64
        );
    }

    #[test]
    fn board_queen_positions() {
        let board = Board::default();

        assert_eq!(
            board.queens,
            0b00001000_00000000_00000000_00000000_00000000_00000000_00000000_00001000u64
        );
    }

    #[test]
    fn board_king_positions() {
        let board = Board::default();

        assert_eq!(
            board.kings,
            0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00010000u64
        );
    }

    #[test]
    fn board_black_positions() {
        let board = Board::default();

        assert_eq!(
            board.black,
            0b11111111_11111111_00000000_00000000_00000000_00000000_00000000_00000000u64
        );
    }

    #[test]
    fn board_white_positions() {
        let board = Board::default();

        assert_eq!(
            board.white,
            0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_11111111u64
        );
    }

    #[test]
    fn board_to_array() {
        let board = Board::default();

        assert_eq!(
            board.to_array(),
            [
                4, 2, 3, 5, 6, 3, 2, 4, //
                1, 1, 1, 1, 1, 1, 1, 1, //
                0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0, 0, //
                -1, -1, -1, -1, -1, -1, -1, -1, //
                -4, -2, -3, -5, -6, -3, -2, -4
            ]
        );
    }

    #[test]
    fn move_encode() {
        let flags = 0b00001101u8;
        let from = 0b00110111u8;
        let to = 0b00101001u8;

        let encoded = Move::new(Square::from_index(from), Square::from_index(to), flags).encode();

        assert_eq!(encoded, 0b11011101_11101001u16);
    }

    #[test]
    fn move_decode() {
        let encoded = 0b11011101_11101001u16;

        let m = Move::decode(encoded);

        assert_eq!(m.flags, 0b00001101u8);
        assert_eq!(m.from.to_index(), 0b00110111u8);
        assert_eq!(m.to.to_index(), 0b00101001u8);
    }
}
