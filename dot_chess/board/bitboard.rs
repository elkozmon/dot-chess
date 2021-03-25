use super::square::{Square, SquareIndex};
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
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

impl core::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rank_str = |rank: u8| -> String {
            format!("{:0>8b}", (self.0 >> (8 * (rank - 1))) & 255)
                .replace("0", " .")
                .replace("1", " x")
                .chars()
                .rev()
                .collect()
        };

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
    const NOT_A_FILE: BitBoard = BitBoard(0xfefefefefefefefe);
    const NOT_H_FILE: BitBoard = BitBoard(0x7f7f7f7f7f7f7f7f);
    const RANK_4: BitBoard = BitBoard(0x00000000ff000000);
    const RANK_5: BitBoard = BitBoard(0x000000ff00000000);

    pub fn square(square_index: SquareIndex) -> Self {
        BitBoard(1 << square_index)
    }

    // Sliding pieces

    pub fn rank_mask(square_index: SquareIndex) -> Self {
        let square = BitBoard::square(square_index).0;

        Self(0xffu64) << (square & 56u64) as i32
    }

    pub fn file_mask(square_index: SquareIndex) -> Self {
        let square = BitBoard::square(square_index).0;

        Self(0x0101010101010101u64) << (square & 7u64) as i32
    }

    pub fn diagonal_mask(square_index: SquareIndex) -> Self {
        let square = BitBoard::square(square_index).0 as i64;

        let maindia = 0x804020100804020i64;
        let diag = 8 * (square & 7) - (square & 56);
        let nort = -diag & (diag >> 31);
        let sout = diag & (-diag >> 31);

        Self(((maindia >> sout) << nort) as u64)
    }

    pub fn anti_diagonal_mask(square_index: SquareIndex) -> Self {
        let square = BitBoard::square(square_index).0 as i64;

        let maindia = 0x0102040810204080i64;
        let diag = 56 - 8 * (square & 7) - (square & 56);
        let nort = -diag & (diag >> 31);
        let sout = diag & (-diag >> 31);

        Self(((maindia >> sout) << nort) as u64)
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

        let attacks = king.east_one() | king.west_one() | king;
        let attacks = attacks.north_one() | attacks.south_one();

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
        (self << 1) & Self::NOT_A_FILE
    }

    pub fn north_east_one(self) -> Self {
        (self << 9) & Self::NOT_A_FILE
    }

    pub fn south_east_one(self) -> Self {
        (self >> 7) & Self::NOT_A_FILE
    }

    pub fn west_one(self) -> Self {
        (self >> 1) & Self::NOT_H_FILE
    }

    pub fn south_west_one(self) -> Self {
        (self >> 9) & Self::NOT_H_FILE
    }

    pub fn north_west_one(self) -> Self {
        (self << 7) & Self::NOT_H_FILE
    }

    pub fn get(&self, index: u8) -> bool {
        ((self.0 >> index) & 1) == 1
    }
}
