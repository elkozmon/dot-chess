use super::square::{Square, SquareIndex, SQUARE_INDEX_RANGE};
use bitintr::Lzcnt;
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

impl core::ops::BitXor for BitBoard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
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

impl core::convert::From<u64> for BitBoard {
    fn from(num: u64) -> Self {
        Self(num)
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

        writeln!(f, "BitBoard 0x{:x}:", self.0);

        for rank in (1..=8).rev() {
            writeln!(f, "{} {}", rank, rank_str(rank))?
        }

        writeln!(f, "  A B C D E F G H")
    }
}

impl BitBoard {
    const LENGTH: i8 = 64;

    const KNIGHT_ATTACKS: [i8; 8] = [6, 15, 17, 10, -6, -15, -17, -10];

    const EMPTY: BitBoard = BitBoard(0);
    const FULL: BitBoard = BitBoard(0xffffffffffffffff);
    const NOT_FILE_A: BitBoard = BitBoard(0xfefefefefefefefe);
    const NOT_FILE_H: BitBoard = BitBoard(0x7f7f7f7f7f7f7f7f);
    const NOT_RANK_1: BitBoard = BitBoard(0x00000000000000ff);
    const NOT_RANK_8: BitBoard = BitBoard(0x00ffffffffffffff);
    const RANK_4: BitBoard = BitBoard(0x00000000ff000000);
    const RANK_5: BitBoard = BitBoard(0x000000ff00000000);

    // General

    pub fn square(square_index: SquareIndex) -> Self {
        BitBoard(1 << square_index)
    }

    pub fn rank_mask(square_index: SquareIndex) -> Self {
        Self(0xffu64) << (square_index as i32 & 56)
    }

    pub fn file_mask(square_index: SquareIndex) -> Self {
        Self(0x0101010101010101u64) << (square_index as i32 & 7)
    }

    pub fn diagonal_mask(square_index: SquareIndex) -> Self {
        let square_index = square_index as i32;

        let diag = 8 * (square_index & 7) - (square_index & 56);
        let nort = -diag & (diag >> 31);
        let sout = diag & (-diag >> 31);

        (Self(0x8040201008040201u64) >> sout) << nort
    }

    pub fn anti_diagonal_mask(square_index: SquareIndex) -> Self {
        let square_index = square_index as i32;

        let diag = 56 - 8 * (square_index & 7) - (square_index & 56);
        let nort = -diag & (diag >> 31);
        let sout = diag & (-diag >> 31);

        (Self(0x0102040810204080u64) >> sout) << nort
    }

    pub fn in_between(from_square: SquareIndex, to_square: SquareIndex) -> Self {
        let from_bb = Self::square(from_square);
        let to_bb = Self::square(to_square);

        let from_clz = from_bb.0.clz() as i32;
        let to_clz = to_bb.0.clz() as i32;

        let between = if from_clz > to_clz {
            !(Self::FULL >> from_clz) & Self::FULL >> to_clz + 1
        } else {
            !(Self::FULL >> to_clz) & Self::FULL >> from_clz + 1
        };

        let rank_bb = Self::rank_mask(from_square);
        if rank_bb & to_bb != Self::EMPTY {
            return rank_bb & between;
        }

        let file_bb = Self::file_mask(from_square);
        if file_bb & to_bb != Self::EMPTY {
            return file_bb & between;
        }

        let diagonal_bb = Self::diagonal_mask(from_square);
        if diagonal_bb & to_bb != Self::EMPTY {
            return diagonal_bb & between;
        }

        let anti_diagonal_bb = Self::anti_diagonal_mask(from_square);
        if anti_diagonal_bb & to_bb != Self::EMPTY {
            return anti_diagonal_bb & between;
        }

        Self::EMPTY
    }

    // Sliding pieces

    pub fn rook_attacks(square_index: SquareIndex) -> Self {
        Self::file_mask(square_index) ^ Self::rank_mask(square_index)
    }

    pub fn bishop_attacks(square_index: SquareIndex) -> Self {
        Self::diagonal_mask(square_index) ^ Self::anti_diagonal_mask(square_index)
    }

    pub fn queen_attacks(square_index: SquareIndex) -> Self {
        Self::rook_attacks(square_index) | Self::bishop_attacks(square_index)
    }

    // Knights

    pub fn knight_attacks(square_index: SquareIndex) -> Self {
        let square_index = square_index as i8;

        Self::KNIGHT_ATTACKS
            .iter()
            .filter_map(|i| match square_index + i {
                i if i >= BitBoard::LENGTH => None,
                i if i < 0 => None,
                i => Some(Self::square(i as u8)),
            })
            .fold(Self::EMPTY, |l, r| l | r)
    }

    // Kings

    pub fn king_attacks(square_index: SquareIndex) -> Self {
        let king = Self::square(square_index);

        let mut attacks = king.east_one() | king.west_one() | king;
        attacks |= attacks.north_one() | attacks.south_one();

        attacks ^ king
    }

    // Pawns

    pub fn white_pawn_east_attacks(white_pawns: Self) -> Self {
        white_pawns.north_east_one()
    }

    pub fn white_pawn_west_attacks(white_pawns: Self) -> Self {
        white_pawns.north_west_one()
    }

    pub fn black_pawn_east_attacks(black_pawns: Self) -> Self {
        black_pawns.south_east_one()
    }

    pub fn black_pawn_west_attacks(black_pawns: Self) -> Self {
        black_pawns.south_west_one()
    }

    pub fn white_pawn_any_attacks(white_pawns: Self) -> Self {
        Self::white_pawn_east_attacks(white_pawns) | Self::white_pawn_west_attacks(white_pawns)
    }

    pub fn white_pawn_double_attacks(white_pawns: Self) -> Self {
        Self::white_pawn_east_attacks(white_pawns) & Self::white_pawn_west_attacks(white_pawns)
    }

    pub fn white_pawn_single_attacks(white_pawns: Self) -> Self {
        Self::white_pawn_east_attacks(white_pawns) ^ Self::white_pawn_west_attacks(white_pawns)
    }

    pub fn black_pawn_any_attacks(black_pawns: Self) -> Self {
        Self::black_pawn_east_attacks(black_pawns) | Self::black_pawn_west_attacks(black_pawns)
    }

    pub fn black_pawn_double_attacks(black_pawns: Self) -> Self {
        Self::black_pawn_east_attacks(black_pawns) & Self::black_pawn_west_attacks(black_pawns)
    }

    pub fn black_pawn_single_attacks(black_pawns: Self) -> Self {
        Self::black_pawn_east_attacks(black_pawns) ^ Self::black_pawn_west_attacks(black_pawns)
    }

    pub fn white_single_push_targets(white_pawns: Self) -> Self {
        white_pawns.north_one()
    }

    pub fn white_double_push_targets(white_pawns: Self) -> Self {
        Self::white_single_push_targets(white_pawns).north_one() & Self::RANK_4
    }

    pub fn black_single_push_targets(black_pawns: Self) -> Self {
        black_pawns.south_one()
    }

    pub fn black_double_push_targets(black_pawns: Self) -> Self {
        Self::black_single_push_targets(black_pawns).south_one() & Self::RANK_5
    }
}

impl BitBoard {
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

    pub fn get(&self, square_index: u8) -> bool {
        ((self.0 >> square_index) & 1) == 1
    }
}

#[cfg(test)]
mod tests {
    use super::super::File;
    use super::super::Rank;
    use super::*;

    #[test]
    fn rank_1_mask() {
        let bb = BitBoard::rank_mask(Square::new(File::H, Rank::_1).index());

        assert_eq!(bb, BitBoard(0xff));
    }

    #[test]
    fn rank_3_mask() {
        let bb = BitBoard::rank_mask(Square::new(File::H, Rank::_3).index());

        assert_eq!(bb, BitBoard(0xff0000));
    }

    #[test]
    fn file_h_mask() {
        let bb = BitBoard::file_mask(Square::new(File::H, Rank::_1).index());

        assert_eq!(bb, BitBoard(0x8080808080808080));
    }

    #[test]
    fn file_d_mask() {
        let bb = BitBoard::file_mask(Square::new(File::D, Rank::_3).index());

        assert_eq!(bb, BitBoard(0x808080808080808));
    }

    #[test]
    fn diagonal_mask_b3() {
        let bb = BitBoard::diagonal_mask(Square::new(File::B, Rank::_3).index());

        assert_eq!(bb, BitBoard(0x4020100804020100));
    }

    #[test]
    fn anti_diagonal_mask_b3() {
        let bb = BitBoard::anti_diagonal_mask(Square::new(File::B, Rank::_3).index());

        assert_eq!(bb, BitBoard(0x1020408));
    }

    #[test]
    fn square_h8() {
        let bb = BitBoard::square(Square::new(File::H, Rank::_8).index());

        assert_eq!(bb, BitBoard(0x8000000000000000));
    }

    #[test]
    fn square_a1() {
        let bb = BitBoard::square(Square::new(File::A, Rank::_1).index());

        assert_eq!(bb, BitBoard(0x1));
    }

    #[test]
    fn square_a2() {
        let bb = BitBoard::square(Square::new(File::A, Rank::_2).index());

        assert_eq!(bb, BitBoard(0x100));
    }

    #[test]
    fn square_b2() {
        let bb = BitBoard::square(Square::new(File::B, Rank::_2).index());

        assert_eq!(bb, BitBoard(0x200));
    }

    #[test]
    fn queen_e6_attacks() {
        let bb = BitBoard::queen_attacks(Square::new(File::E, Rank::_6).index());

        assert_eq!(bb, BitBoard(0x5438ef3854921110));
    }

    #[test]
    fn rook_e6_attacks() {
        let bb = BitBoard::rook_attacks(Square::new(File::E, Rank::_6).index());

        assert_eq!(bb, BitBoard(0x1010ef1010101010));
    }

    #[test]
    fn bishop_e6_attacks() {
        let bb = BitBoard::bishop_attacks(Square::new(File::E, Rank::_6).index());

        assert_eq!(bb, BitBoard(0x4428002844820100));
    }

    #[test]
    fn king_a1_attacks() {
        let bb = BitBoard::king_attacks(Square::new(File::A, Rank::_1).index());

        assert_eq!(bb, BitBoard(0x302));
    }

    #[test]
    fn king_h7_attacks() {
        let bb = BitBoard::king_attacks(Square::new(File::H, Rank::_7).index());

        assert_eq!(bb, BitBoard(0xc040c00000000000));
    }

    #[test]
    fn king_d3_attacks() {
        let bb = BitBoard::king_attacks(Square::new(File::D, Rank::_3).index());

        assert_eq!(bb, BitBoard(0x1c141c00));
    }

    #[test]
    fn white_pawn_any_attacks() {
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
            white_pawns |= BitBoard::square(square.index());
        }

        let bb = BitBoard::white_pawn_any_attacks(white_pawns);

        assert_eq!(bb, BitBoard(0x4000000050051e00));
    }

    #[test]
    fn white_pawn_east_attacks() {
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
            white_pawns |= BitBoard::square(square.index());
        }

        let bb = BitBoard::white_pawn_east_attacks(white_pawns);

        assert_eq!(bb, BitBoard(0x40041a00));
    }

    #[test]
    fn white_pawn_west_attacks() {
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
            white_pawns |= BitBoard::square(square.index());
        }

        let bb = BitBoard::white_pawn_west_attacks(white_pawns);

        assert_eq!(bb, BitBoard(0x4000000010010600));
    }

    #[test]
    fn white_pawn_single_attacks() {
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
            white_pawns |= BitBoard::square(square.index());
        }

        let bb = BitBoard::white_pawn_single_attacks(white_pawns);

        assert_eq!(bb, BitBoard(0x4000000050051c00));
    }

    #[test]
    fn white_pawn_double_attacks() {
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
            white_pawns |= BitBoard::square(square.index());
        }

        let bb = BitBoard::white_pawn_double_attacks(white_pawns);

        assert_eq!(bb, BitBoard(0x200));
    }

    #[test]
    fn black_pawn_any_attacks() {
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
            black_pawns |= BitBoard::square(square.index());
        }

        let bb = BitBoard::black_pawn_any_attacks(black_pawns);

        assert_eq!(bb, BitBoard(0x1e050050000040));
    }

    #[test]
    fn black_pawn_east_attacks() {
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
            black_pawns |= BitBoard::square(square.index());
        }

        let bb = BitBoard::black_pawn_east_attacks(black_pawns);

        assert_eq!(bb, BitBoard(0x1a040040000000));
    }

    #[test]
    fn black_pawn_west_attacks() {
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
            black_pawns |= BitBoard::square(square.index());
        }

        let bb = BitBoard::black_pawn_west_attacks(black_pawns);

        assert_eq!(bb, BitBoard(0x6010010000040));
    }

    #[test]
    fn black_pawn_single_attacks() {
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
            black_pawns |= BitBoard::square(square.index());
        }

        let bb = BitBoard::black_pawn_single_attacks(black_pawns);

        assert_eq!(bb, BitBoard(0x1c050050000040));
    }

    #[test]
    fn black_pawn_double_attacks() {
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
            black_pawns |= BitBoard::square(square.index());
        }

        let bb = BitBoard::black_pawn_double_attacks(black_pawns);

        assert_eq!(bb, BitBoard(0x2000000000000));
    }

    #[test]
    fn nort_one_h1() {
        let bb = BitBoard::square(Square::new(File::H, Rank::_1).index()).north_one();

        assert_eq!(bb, BitBoard(0x8000));
    }

    #[test]
    fn nort_one_a8() {
        let bb = BitBoard::square(Square::new(File::A, Rank::_8).index()).north_one();

        assert_eq!(bb, BitBoard::EMPTY);
    }

    #[test]
    fn nort_east_one_g1() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_1).index()).north_east_one();

        assert_eq!(bb, BitBoard(0x8000));
    }

    #[test]
    fn north_west_one_g1() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_1).index()).north_west_one();

        assert_eq!(bb, BitBoard(0x2000));
    }

    #[test]
    fn east_one_g2() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_2).index()).east_one();

        assert_eq!(bb, BitBoard(0x8000));
    }

    #[test]
    fn west_one_g2() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_2).index()).west_one();

        assert_eq!(bb, BitBoard(0x2000));
    }

    #[test]
    fn south_one_g2() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_2).index()).south_one();

        assert_eq!(bb, BitBoard(0x40));
    }

    #[test]
    fn south_one_g1() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_1).index()).south_one();

        assert_eq!(bb, BitBoard::EMPTY);
    }

    #[test]
    fn south_west_one_g2() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_2).index()).south_west_one();

        assert_eq!(bb, BitBoard(0x20));
    }

    #[test]
    fn south_east_one_g2() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_2).index()).south_east_one();

        assert_eq!(bb, BitBoard(0x80));
    }

    #[test]
    fn get_g2_true() {
        let bb = BitBoard::square(Square::new(File::G, Rank::_2).index());

        assert_eq!(bb.get(Square::new(File::G, Rank::_2).index()), true);
    }

    #[test]
    fn get_g2_false() {
        let bb = !BitBoard::square(Square::new(File::G, Rank::_2).index());

        assert_eq!(bb.get(Square::new(File::G, Rank::_2).index()), false);
    }

    #[test]
    fn neg_bitboard() {
        let bb = !BitBoard::square(Square::new(File::G, Rank::_2).index());

        assert_eq!(bb, BitBoard(0xffffffffffffbfff));
    }

    #[test]
    fn in_between_a1_h8() {
        let bb = BitBoard::in_between(
            Square::new(File::A, Rank::_1).index(),
            Square::new(File::H, Rank::_8).index(),
        );

        assert_eq!(bb, BitBoard(0x40201008040200));
    }

    #[test]
    fn in_between_h8_a1() {
        let bb = BitBoard::in_between(
            Square::new(File::H, Rank::_8).index(),
            Square::new(File::A, Rank::_1).index(),
        );

        assert_eq!(bb, BitBoard(0x40201008040200));
    }

    #[test]
    fn in_between_g8_a1() {
        let bb = BitBoard::in_between(
            Square::new(File::G, Rank::_8).index(),
            Square::new(File::A, Rank::_1).index(),
        );

        assert_eq!(bb, BitBoard::EMPTY);
    }

    #[test]
    fn in_between_a1_a8() {
        let bb = BitBoard::in_between(
            Square::new(File::A, Rank::_1).index(),
            Square::new(File::A, Rank::_8).index(),
        );

        assert_eq!(bb, BitBoard(0x1010101010100));
    }

    #[test]
    fn in_between_b4_g4() {
        let bb = BitBoard::in_between(
            Square::new(File::B, Rank::_4).index(),
            Square::new(File::G, Rank::_4).index(),
        );

        assert_eq!(bb, BitBoard(0x3c000000));
    }
}
