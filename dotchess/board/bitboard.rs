use super::{square::Square, Direction, File, Rank};
use bitintr::{Blsfill, Blsr, Lzcnt, Tzcnt};
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct BitBoard(u64);

impl core::ops::BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl core::ops::BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl core::ops::BitAnd<bool> for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: bool) -> Self::Output {
        if !rhs {
            return Self::EMPTY;
        }

        return self;
    }
}

impl core::ops::BitAndAssign<bool> for BitBoard {
    fn bitand_assign(&mut self, rhs: bool) {
        if !rhs {
            self.0 = Self::EMPTY.0;
        }
    }
}

impl core::ops::BitXor for BitBoard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl core::ops::BitXorAssign for BitBoard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl core::ops::Shl<i32> for BitBoard {
    type Output = Self;

    fn shl(self, rhs: i32) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl core::ops::Shr<i32> for BitBoard {
    type Output = Self;

    fn shr(self, rhs: i32) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl core::ops::Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl core::convert::From<Square> for BitBoard {
    fn from(square: Square) -> Self {
        let square_index: i32 = <Square as Into<u8>>::into(square) as i32;

        Self(1) << square_index
    }
}

impl core::convert::From<Rank> for BitBoard {
    fn from(rank: Rank) -> Self {
        match rank {
            Rank::_1 => Self::RANK_1,
            Rank::_2 => Self::RANK_2,
            Rank::_3 => Self::RANK_3,
            Rank::_4 => Self::RANK_4,
            Rank::_5 => Self::RANK_5,
            Rank::_6 => Self::RANK_6,
            Rank::_7 => Self::RANK_7,
            Rank::_8 => Self::RANK_8,
        }
    }
}

impl core::convert::From<File> for BitBoard {
    fn from(file: File) -> Self {
        match file {
            File::A => Self::FILE_A,
            File::B => Self::FILE_B,
            File::C => Self::FILE_C,
            File::D => Self::FILE_D,
            File::E => Self::FILE_E,
            File::F => Self::FILE_F,
            File::G => Self::FILE_G,
            File::H => Self::FILE_H,
        }
    }
}

impl core::convert::From<u64> for BitBoard {
    fn from(num: u64) -> Self {
        Self(num)
    }
}

impl core::convert::Into<u64> for BitBoard {
    fn into(self) -> u64 {
        self.0
    }
}

impl core::iter::IntoIterator for BitBoard {
    type Item = Square;

    type IntoIter = BitBoardIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        BitBoardIntoIterator(self)
    }
}

impl Blsfill for BitBoard {
    fn blsfill(self) -> Self {
        Self(self.0.blsfill())
    }
}

impl Blsr for BitBoard {
    fn blsr(self) -> Self {
        Self(self.0.blsr())
    }
}

pub struct BitBoardIntoIterator(BitBoard);

impl core::iter::Iterator for BitBoardIntoIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        Some(self.0.pop_square())
    }
}

#[cfg(feature = "std")]
impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rank_str = |rank: u8| -> String {
            format!("{:0>8b}", (self.0 >> (8 * (rank - 1))) & 255)
                .replace("0", " .")
                .replace("1", " x")
                .chars()
                .rev()
                .collect()
        };

        writeln!(f, "BitBoard 0x{:x}:", self.0)?;

        for rank in (1..=8).rev() {
            writeln!(f, "{} {}", rank, rank_str(rank))?;
        }

        writeln!(f, "  A B C D E F G H")
    }
}

impl BitBoard {
    pub const RANK_1: Self = Self(0x00000000000000ff);
    pub const RANK_2: Self = Self(0x000000000000ff00);
    pub const RANK_3: Self = Self(0x0000000000ff0000);
    pub const RANK_4: Self = Self(0x00000000ff000000);
    pub const RANK_5: Self = Self(0x000000ff00000000);
    pub const RANK_6: Self = Self(0x0000ff0000000000);
    pub const RANK_7: Self = Self(0x00ff000000000000);
    pub const RANK_8: Self = Self(0xff00000000000000);

    pub const FILE_A: Self = Self(0x0101010101010101);
    pub const FILE_B: Self = Self(0x0202020202020202);
    pub const FILE_C: Self = Self(0x0404040404040404);
    pub const FILE_D: Self = Self(0x0808080808080808);
    pub const FILE_E: Self = Self(0x1010101010101010);
    pub const FILE_F: Self = Self(0x2020202020202020);
    pub const FILE_G: Self = Self(0x4040404040404040);
    pub const FILE_H: Self = Self(0x8080808080808080);

    pub const EMPTY: Self = Self(0);
    pub const FULL: Self = Self(0xffffffffffffffff);

    pub const NOT_FILE_A: Self = Self(0xfefefefefefefefe);
    pub const NOT_FILE_B: Self = Self(0xfdfdfdfdfdfdfdfd);
    pub const NOT_FILE_AB: Self = Self(0xfcfcfcfcfcfcfcfc);
    pub const NOT_FILE_G: Self = Self(0xbfbfbfbfbfbfbfbf);
    pub const NOT_FILE_H: Self = Self(0x7f7f7f7f7f7f7f7f);
    pub const NOT_FILE_GH: Self = Self(0x3f3f3f3f3f3f3f3f);

    // General

    pub fn square(square: Square) -> Self {
        <BitBoard as From<Square>>::from(square)
    }

    pub fn positive(square: Square) -> Self {
        !Self::square(square).blsfill()
    }

    pub fn negative(square: Square) -> Self {
        Self::square(square).blsfill() >> 1
    }

    pub fn rank_mask(square: Square) -> Self {
        let square_index: i32 = <Square as Into<u8>>::into(square) as i32;

        Self(0xffu64) << (square_index & 56)
    }

    pub fn file_mask(square: Square) -> Self {
        let square_index: i32 = <Square as Into<u8>>::into(square) as i32;

        Self(0x0101010101010101u64) << (square_index & 7)
    }

    pub fn diagonal_mask(square: Square) -> Self {
        let square_index: i32 = <Square as Into<u8>>::into(square) as i32;

        let diag = 8 * (square_index & 7) - (square_index & 56);
        let nort = -diag & (diag >> 31);
        let sout = diag & (-diag >> 31);

        (Self(0x8040201008040201u64) >> sout) << nort
    }

    pub fn anti_diagonal_mask(square: Square) -> Self {
        let square_index: i32 = <Square as Into<u8>>::into(square) as i32;

        let diag = 56 - 8 * (square_index & 7) - (square_index & 56);
        let nort = -diag & (diag >> 31);
        let sout = diag & (-diag >> 31);

        (Self(0x0102040810204080u64) >> sout) << nort
    }

    pub fn in_between_mask(from: Square, to: Square) -> Self {
        let from_bb = Self::square(from);
        let to_bb = Self::square(to);

        let between = ((from_bb.blsfill() ^ to_bb.blsfill()) >> 1).blsr();

        let rank_bb = Self::rank_mask(from);
        if rank_bb & to_bb != Self::EMPTY {
            return rank_bb & between;
        }

        let file_bb = Self::file_mask(from);
        if file_bb & to_bb != Self::EMPTY {
            return file_bb & between;
        }

        let diagonal_bb = Self::diagonal_mask(from);
        if diagonal_bb & to_bb != Self::EMPTY {
            return diagonal_bb & between;
        }

        let anti_diagonal_bb = Self::anti_diagonal_mask(from);
        if anti_diagonal_bb & to_bb != Self::EMPTY {
            return anti_diagonal_bb & between;
        }

        Self::EMPTY
    }

    pub fn ray_mask(square: Square, direction: Direction) -> Self {
        match direction {
            Direction::North => Self::file_mask(square) & Self::positive(square),
            Direction::NorthEast => Self::diagonal_mask(square) & Self::positive(square),
            Direction::East => Self::rank_mask(square) & Self::positive(square),
            Direction::SouthEast => Self::anti_diagonal_mask(square) & Self::negative(square),
            Direction::South => Self::file_mask(square) & Self::negative(square),
            Direction::SouthWest => Self::diagonal_mask(square) & Self::negative(square),
            Direction::West => Self::rank_mask(square) & Self::negative(square),
            Direction::NorthWest => Self::anti_diagonal_mask(square) & Self::positive(square),
        }
    }

    pub fn rook_attacks_mask(square: Square) -> Self {
        Self::file_mask(square) ^ Self::rank_mask(square)
    }

    pub fn bishop_attacks_mask(square: Square) -> Self {
        Self::diagonal_mask(square) ^ Self::anti_diagonal_mask(square)
    }

    pub fn queen_attacks_mask(square: Square) -> Self {
        Self::rook_attacks_mask(square) | Self::bishop_attacks_mask(square)
    }

    pub fn knight_attacks_mask(square: Square) -> Self {
        let mut mask = Self::EMPTY;

        let knight = Self::square(square);

        mask |= (knight << 17) & Self::NOT_FILE_A; // NoNoEa
        mask |= (knight << 10) & Self::NOT_FILE_AB; // NoEaEa
        mask |= (knight >> 6) & Self::NOT_FILE_AB; // SoEaEa
        mask |= (knight >> 15) & Self::NOT_FILE_A; // SoSoEa
        mask |= (knight << 15) & Self::NOT_FILE_H; // NoNoWe
        mask |= (knight << 6) & Self::NOT_FILE_GH; // NoWeWe
        mask |= (knight >> 10) & Self::NOT_FILE_GH; // SoWeWe
        mask |= (knight >> 17) & Self::NOT_FILE_H; // SoSoWe

        mask
    }

    pub fn king_attacks_mask(square: Square) -> Self {
        let king = Self::square(square);

        let mut attacks = king.east_one() | king.west_one() | king;
        attacks |= attacks.north_one() | attacks.south_one();

        attacks ^ king
    }

    // Pawns

    pub fn white_pawn_east_attacks_mask(&self) -> Self {
        self.north_east_one()
    }

    pub fn white_pawn_west_attacks_mask(&self) -> Self {
        self.north_west_one()
    }

    pub fn black_pawn_east_attacks_mask(&self) -> Self {
        self.south_east_one()
    }

    pub fn black_pawn_west_attacks_mask(&self) -> Self {
        self.south_west_one()
    }

    pub fn white_pawn_any_attacks_mask(&self) -> Self {
        self.white_pawn_east_attacks_mask() | self.white_pawn_west_attacks_mask()
    }

    pub fn white_pawn_double_attacks_mask(&self) -> Self {
        self.white_pawn_east_attacks_mask() & self.white_pawn_west_attacks_mask()
    }

    pub fn white_pawn_single_attacks_mask(&self) -> Self {
        self.white_pawn_east_attacks_mask() ^ self.white_pawn_west_attacks_mask()
    }

    pub fn black_pawn_any_attacks_mask(&self) -> Self {
        self.black_pawn_east_attacks_mask() | self.black_pawn_west_attacks_mask()
    }

    pub fn black_pawn_double_attacks_mask(&self) -> Self {
        self.black_pawn_east_attacks_mask() & self.black_pawn_west_attacks_mask()
    }

    pub fn black_pawn_single_attacks_mask(&self) -> Self {
        self.black_pawn_east_attacks_mask() ^ self.black_pawn_west_attacks_mask()
    }
}

impl BitBoard {
    pub fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    pub fn not_empty(self) -> bool {
        !self.is_empty()
    }

    pub fn north_one(self) -> Self {
        self << 8
    }

    pub fn south_one(self) -> Self {
        self >> 8
    }

    pub fn east_one(self) -> Self {
        (self << 1) & Self::NOT_FILE_A
    }

    pub fn north_east_one(self) -> Self {
        (self << 9) & Self::NOT_FILE_A
    }

    pub fn south_east_one(self) -> Self {
        (self >> 7) & Self::NOT_FILE_A
    }

    pub fn west_one(self) -> Self {
        (self >> 1) & Self::NOT_FILE_H
    }

    pub fn south_west_one(self) -> Self {
        (self >> 9) & Self::NOT_FILE_H
    }

    pub fn north_west_one(self) -> Self {
        (self << 7) & Self::NOT_FILE_H
    }

    pub fn get(&self, square: Square) -> bool {
        let index: u8 = square.into();

        ((self.0 >> index) & 1) == 1
    }

    pub fn pop_square(&mut self) -> Square {
        assert!(self.not_empty());

        let square = self.bit_scan_forward();
        let index: u8 = square.into();

        self.0 ^= 1 << index;

        square
    }

    pub fn bit_scan_forward(&self) -> Square {
        assert_ne!(self.0, 0);

        (self.0.tzcnt() as u8).into()
    }

    pub fn bit_scan_reverse(&self) -> Square {
        assert_ne!(self.0, 0);

        (63 - self.0.lzcnt() as u8).into()
    }
}

#[cfg(test)]
mod tests {
    use super::super::File;
    use super::super::Rank;
    use super::*;

    #[test]
    fn rank_1_mask() {
        let bb = BitBoard::rank_mask(Square::new(File::H, Rank::_1));

        assert_eq!(bb, BitBoard::RANK_1);
    }

    #[test]
    fn rank_2_mask() {
        let bb = BitBoard::rank_mask(Square::new(File::H, Rank::_2));

        assert_eq!(bb, BitBoard::RANK_2);
    }

    #[test]
    fn rank_3_mask() {
        let bb = BitBoard::rank_mask(Square::new(File::H, Rank::_3));

        assert_eq!(bb, BitBoard::RANK_3);
    }

    #[test]
    fn rank_4_mask() {
        let bb = BitBoard::rank_mask(Square::new(File::H, Rank::_4));

        assert_eq!(bb, BitBoard::RANK_4);
    }

    #[test]
    fn rank_5_mask() {
        let bb = BitBoard::rank_mask(Square::new(File::H, Rank::_5));

        assert_eq!(bb, BitBoard::RANK_5);
    }

    #[test]
    fn rank_6_mask() {
        let bb = BitBoard::rank_mask(Square::new(File::H, Rank::_6));

        assert_eq!(bb, BitBoard::RANK_6);
    }

    #[test]
    fn rank_7_mask() {
        let bb = BitBoard::rank_mask(Square::new(File::H, Rank::_7));

        assert_eq!(bb, BitBoard::RANK_7);
    }

    #[test]
    fn rank_8_mask() {
        let bb = BitBoard::rank_mask(Square::new(File::H, Rank::_8));

        assert_eq!(bb, BitBoard::RANK_8);
    }

    #[test]
    fn file_a_mask() {
        let bb = BitBoard::file_mask(Square::new(File::A, Rank::_3));

        assert_eq!(bb, BitBoard::FILE_A);
    }

    #[test]
    fn file_b_mask() {
        let bb = BitBoard::file_mask(Square::new(File::B, Rank::_3));

        assert_eq!(bb, BitBoard::FILE_B);
    }

    #[test]
    fn file_c_mask() {
        let bb = BitBoard::file_mask(Square::new(File::C, Rank::_3));

        assert_eq!(bb, BitBoard::FILE_C);
    }

    #[test]
    fn file_d_mask() {
        let bb = BitBoard::file_mask(Square::new(File::D, Rank::_3));

        assert_eq!(bb, BitBoard::FILE_D);
    }

    #[test]
    fn file_e_mask() {
        let bb = BitBoard::file_mask(Square::new(File::E, Rank::_3));

        assert_eq!(bb, BitBoard::FILE_E);
    }

    #[test]
    fn file_f_mask() {
        let bb = BitBoard::file_mask(Square::new(File::F, Rank::_3));

        assert_eq!(bb, BitBoard::FILE_F);
    }

    #[test]
    fn file_g_mask() {
        let bb = BitBoard::file_mask(Square::new(File::G, Rank::_3));

        assert_eq!(bb, BitBoard::FILE_G);
    }

    #[test]
    fn file_h_mask() {
        let bb = BitBoard::file_mask(Square::new(File::H, Rank::_1));

        assert_eq!(bb, BitBoard::FILE_H);
    }

    #[test]
    fn diagonal_mask_b3() {
        let bb = BitBoard::diagonal_mask(Square::new(File::B, Rank::_3));

        assert_eq!(bb, BitBoard(0x4020100804020100));
    }

    #[test]
    fn anti_diagonal_mask_b3() {
        let bb = BitBoard::anti_diagonal_mask(Square::new(File::B, Rank::_3));

        assert_eq!(bb, BitBoard(0x1020408));
    }

    #[test]
    fn square_h8() {
        let bb = BitBoard::square(Square::new(File::H, Rank::_8));

        assert_eq!(bb, BitBoard(0x8000000000000000));
    }

    #[test]
    fn square_a1() {
        let bb = BitBoard::square(Square::new(File::A, Rank::_1));

        assert_eq!(bb, BitBoard(0x1));
    }

    #[test]
    fn square_a2() {
        let bb = BitBoard::square(Square::new(File::A, Rank::_2));

        assert_eq!(bb, BitBoard(0x100));
    }

    #[test]
    fn square_b2() {
        let bb = BitBoard::square(Square::new(File::B, Rank::_2));

        assert_eq!(bb, BitBoard(0x200));
    }

    #[test]
    fn queen_e6_attacks_mask() {
        let bb = BitBoard::queen_attacks_mask(Square::new(File::E, Rank::_6));

        assert_eq!(bb, BitBoard(0x5438ef3854921110));
    }

    #[test]
    fn rook_e6_attacks_mask() {
        let bb = BitBoard::rook_attacks_mask(Square::new(File::E, Rank::_6));

        assert_eq!(bb, BitBoard(0x1010ef1010101010));
    }

    #[test]
    fn bishop_e6_attacks_mask() {
        let bb = BitBoard::bishop_attacks_mask(Square::new(File::E, Rank::_6));

        assert_eq!(bb, BitBoard(0x4428002844820100));
    }

    #[test]
    fn king_a1_attacks_mask() {
        let bb = BitBoard::king_attacks_mask(Square::new(File::A, Rank::_1));

        assert_eq!(bb, BitBoard(0x302));
    }

    #[test]
    fn king_h7_attacks_mask() {
        let bb = BitBoard::king_attacks_mask(Square::new(File::H, Rank::_7));

        assert_eq!(bb, BitBoard(0xc040c00000000000));
    }

    #[test]
    fn king_d3_attacks_mask() {
        let bb = BitBoard::king_attacks_mask(Square::new(File::D, Rank::_3));

        assert_eq!(bb, BitBoard(0x1c141c00));
    }

    #[test]
    fn white_pawn_any_attacks_mask() {
        let mut white_pawns = BitBoard::EMPTY;

        let squares = [
            Square::new(File::A, Rank::_1),
            Square::new(File::B, Rank::_2),
            Square::new(File::C, Rank::_1),
            Square::new(File::D, Rank::_1),
            Square::new(File::F, Rank::_3),
            Square::new(File::H, Rank::_7),
        ];

        for square in squares.iter() {
            white_pawns |= BitBoard::square(*square);
        }

        assert_eq!(
            white_pawns.white_pawn_any_attacks_mask(),
            BitBoard(0x4000000050051e00)
        );
    }

    #[test]
    fn white_pawn_east_attacks_mask() {
        let mut white_pawns = BitBoard::EMPTY;

        let squares = [
            Square::new(File::A, Rank::_1),
            Square::new(File::B, Rank::_2),
            Square::new(File::C, Rank::_1),
            Square::new(File::D, Rank::_1),
            Square::new(File::F, Rank::_3),
            Square::new(File::H, Rank::_7),
        ];

        for square in squares.iter() {
            white_pawns |= BitBoard::square(*square);
        }

        assert_eq!(
            white_pawns.white_pawn_east_attacks_mask(),
            BitBoard(0x40041a00)
        );
    }

    #[test]
    fn white_pawn_west_attacks_mask() {
        let mut white_pawns = BitBoard::EMPTY;

        let squares = [
            Square::new(File::A, Rank::_1),
            Square::new(File::B, Rank::_2),
            Square::new(File::C, Rank::_1),
            Square::new(File::D, Rank::_1),
            Square::new(File::F, Rank::_3),
            Square::new(File::H, Rank::_7),
        ];

        for square in squares.iter() {
            white_pawns |= BitBoard::square(*square);
        }

        assert_eq!(
            white_pawns.white_pawn_west_attacks_mask(),
            BitBoard(0x4000000010010600)
        );
    }

    #[test]
    fn white_pawn_single_attacks_mask() {
        let mut white_pawns = BitBoard::EMPTY;

        let squares = [
            Square::new(File::A, Rank::_1),
            Square::new(File::B, Rank::_2),
            Square::new(File::C, Rank::_1),
            Square::new(File::D, Rank::_1),
            Square::new(File::F, Rank::_3),
            Square::new(File::H, Rank::_7),
        ];

        for square in squares.iter() {
            white_pawns |= BitBoard::square(*square);
        }

        assert_eq!(
            white_pawns.white_pawn_single_attacks_mask(),
            BitBoard(0x4000000050051c00)
        );
    }

    #[test]
    fn white_pawn_double_attacks_mask() {
        let mut white_pawns = BitBoard::EMPTY;

        let squares = [
            Square::new(File::A, Rank::_1),
            Square::new(File::B, Rank::_2),
            Square::new(File::C, Rank::_1),
            Square::new(File::D, Rank::_1),
            Square::new(File::F, Rank::_3),
            Square::new(File::H, Rank::_7),
        ];

        for square in squares.iter() {
            white_pawns |= BitBoard::square(*square);
        }

        assert_eq!(
            white_pawns.white_pawn_double_attacks_mask(),
            BitBoard(0x200)
        );
    }

    #[test]
    fn black_pawn_any_attacks_mask() {
        let mut black_pawns = BitBoard::EMPTY;

        let squares = [
            Square::new(File::A, Rank::_8),
            Square::new(File::B, Rank::_7),
            Square::new(File::C, Rank::_8),
            Square::new(File::D, Rank::_8),
            Square::new(File::F, Rank::_5),
            Square::new(File::H, Rank::_2),
        ];

        for square in squares.iter() {
            black_pawns |= BitBoard::square(*square);
        }

        assert_eq!(
            black_pawns.black_pawn_any_attacks_mask(),
            BitBoard(0x1e050050000040)
        );
    }

    #[test]
    fn black_pawn_east_attacks_mask() {
        let mut black_pawns = BitBoard::EMPTY;

        let squares = [
            Square::new(File::A, Rank::_8),
            Square::new(File::B, Rank::_7),
            Square::new(File::C, Rank::_8),
            Square::new(File::D, Rank::_8),
            Square::new(File::F, Rank::_5),
            Square::new(File::H, Rank::_2),
        ];

        for square in squares.iter() {
            black_pawns |= BitBoard::square(*square);
        }

        assert_eq!(
            black_pawns.black_pawn_east_attacks_mask(),
            BitBoard(0x1a040040000000)
        );
    }

    #[test]
    fn black_pawn_west_attacks_mask() {
        let mut black_pawns = BitBoard::EMPTY;

        let squares = [
            Square::new(File::A, Rank::_8),
            Square::new(File::B, Rank::_7),
            Square::new(File::C, Rank::_8),
            Square::new(File::D, Rank::_8),
            Square::new(File::F, Rank::_5),
            Square::new(File::H, Rank::_2),
        ];

        for square in squares.iter() {
            black_pawns |= BitBoard::square(*square);
        }

        assert_eq!(
            black_pawns.black_pawn_west_attacks_mask(),
            BitBoard(0x6010010000040)
        );
    }

    #[test]
    fn black_pawn_single_attacks_mask() {
        let mut black_pawns = BitBoard::EMPTY;

        let squares = [
            Square::new(File::A, Rank::_8),
            Square::new(File::B, Rank::_7),
            Square::new(File::C, Rank::_8),
            Square::new(File::D, Rank::_8),
            Square::new(File::F, Rank::_5),
            Square::new(File::H, Rank::_2),
        ];

        for square in squares.iter() {
            black_pawns |= BitBoard::square(*square);
        }

        assert_eq!(
            black_pawns.black_pawn_single_attacks_mask(),
            BitBoard(0x1c050050000040)
        );
    }

    #[test]
    fn black_pawn_double_attacks_mask() {
        let mut black_pawns = BitBoard::EMPTY;

        let squares = [
            Square::new(File::A, Rank::_8),
            Square::new(File::B, Rank::_7),
            Square::new(File::C, Rank::_8),
            Square::new(File::D, Rank::_8),
            Square::new(File::F, Rank::_5),
            Square::new(File::H, Rank::_2),
        ];

        for square in squares.iter() {
            black_pawns |= BitBoard::square(*square);
        }

        assert_eq!(
            black_pawns.black_pawn_double_attacks_mask(),
            BitBoard(0x2000000000000)
        );
    }

    #[test]
    fn north_one_h1() {
        let bb = BitBoard::square(Square::new(File::H, Rank::_1)).north_one();

        assert_eq!(bb, BitBoard(0x8000));
    }

    #[test]
    fn north_one_a8() {
        let bb = BitBoard::square(Square::new(File::A, Rank::_8)).north_one();

        assert_eq!(bb, BitBoard::EMPTY);
    }

    #[test]
    fn north_east_one_g1() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_1)).north_east_one();

        assert_eq!(bb, BitBoard(0x8000));
    }

    #[test]
    fn north_west_one_g1() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_1)).north_west_one();

        assert_eq!(bb, BitBoard(0x2000));
    }

    #[test]
    fn east_one_g2() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_2)).east_one();

        assert_eq!(bb, BitBoard(0x8000));
    }

    #[test]
    fn west_one_g2() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_2)).west_one();

        assert_eq!(bb, BitBoard(0x2000));
    }

    #[test]
    fn south_one_g2() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_2)).south_one();

        assert_eq!(bb, BitBoard(0x40));
    }

    #[test]
    fn south_one_g1() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_1)).south_one();

        assert_eq!(bb, BitBoard::EMPTY);
    }

    #[test]
    fn south_west_one_g2() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_2)).south_west_one();

        assert_eq!(bb, BitBoard(0x20));
    }

    #[test]
    fn south_east_one_g2() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_2)).south_east_one();

        assert_eq!(bb, BitBoard(0x80));
    }

    #[test]
    fn get_g2_true() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_2));

        assert_eq!(bb.get(Square::new(File::G, Rank::_2)), true);
    }

    #[test]
    fn get_g2_false() {
        let bb = !BitBoard::square(Square::new(File::G, Rank::_2));

        assert_eq!(bb.get(Square::new(File::G, Rank::_2)), false);
    }

    #[test]
    fn neg_bitboard() {
        let bb = !BitBoard::square(Square::new(File::G, Rank::_2));

        assert_eq!(bb, BitBoard(0xffffffffffffbfff));
    }

    #[test]
    fn ray_mask_sw_g5() {
        let bb = BitBoard::ray_mask(Square::new(File::G, Rank::_5), Direction::SouthWest);

        assert_eq!(bb, BitBoard(0x20100804));
    }

    #[test]
    fn ray_mask_nw_g5() {
        let bb = BitBoard::ray_mask(Square::new(File::G, Rank::_5), Direction::NorthWest);

        assert_eq!(bb, BitBoard(0x810200000000000));
    }

    #[test]
    fn in_between_mask_a1_h8() {
        let bb = BitBoard::in_between_mask(
            Square::new(File::A, Rank::_1),
            Square::new(File::H, Rank::_8),
        );

        assert_eq!(bb, BitBoard(0x40201008040200));
    }

    #[test]
    fn in_between_mask_h8_a1() {
        let bb = BitBoard::in_between_mask(
            Square::new(File::H, Rank::_8),
            Square::new(File::A, Rank::_1),
        );

        assert_eq!(bb, BitBoard(0x40201008040200));
    }

    #[test]
    fn in_between_mask_g8_a1() {
        let bb = BitBoard::in_between_mask(
            Square::new(File::G, Rank::_8),
            Square::new(File::A, Rank::_1),
        );

        assert_eq!(bb, BitBoard::EMPTY);
    }

    #[test]
    fn in_between_mask_a1_a8() {
        let bb = BitBoard::in_between_mask(
            Square::new(File::A, Rank::_1),
            Square::new(File::A, Rank::_8),
        );

        assert_eq!(bb, BitBoard(0x1010101010100));
    }

    #[test]
    fn in_between_mask_b4_g4() {
        let bb = BitBoard::in_between_mask(
            Square::new(File::B, Rank::_4),
            Square::new(File::G, Rank::_4),
        );

        assert_eq!(bb, BitBoard(0x3c000000));
    }

    #[test]
    fn pop_square() {
        let square: Square = 18.into();
        let mut bb = BitBoard::square(square);

        assert_eq!(bb.pop_square(), square);
        assert!(bb.is_empty());
    }
}
