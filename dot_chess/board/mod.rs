mod bitboard;
mod direction;
mod event;
mod file;
mod piece;
mod ply;
mod rank;
mod side;
mod square;

use self::bitboard::BitBoard;
use self::square::SQUARE_INDEX_RANGE;
use crate::dot_chess::Error;
use ink_storage::collections::HashMap;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use ink_storage::Vec;
use scale::{Decode, Encode};

pub use direction::Direction;
pub use event::Event;
pub use file::File;
pub use piece::Piece;
pub use ply::{Flags as PlyFlags, Ply};
pub use rank::Rank;
pub use side::Side;
pub use square::{Square, SquareIndex};

/// Board flags bit mask
///
/// 1 << 0  En passant open at file A
/// 1 << 1  En passant open at file B
/// 1 << 2  En passant open at file C
/// 1 << 3  En passant open at file D
/// 1 << 4  En passant open at file E
/// 1 << 5  En passant open at file F
/// 1 << 6  En passant open at file G
/// 1 << 7  En passant open at file H
/// 1 << 8  White Queen Castling Right
/// 1 << 9  White King Castling Right
/// 1 << 10 Black Queen Castling Right
/// 1 << 11 Black King Castling Right
/// 1 << 12 Whites Turn
#[derive(Copy, Clone, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, scale_info::TypeInfo, StorageLayout)
)]
pub struct Flags(u16);

impl Flags {
    const WHITES_TURN_INDEX: usize = 12;

    pub fn default() -> Self {
        Self(0b11111000_00000000u16)
    }

    fn get_bit(&self, bit: usize) -> bool {
        ((self.0 >> bit) & 1u16) == 1
    }

    fn set_bit(&mut self, bit: usize, to: bool) -> () {
        self.0 = (self.0 & !(1u16 << bit)) | ((to as u16) << bit);
    }

    fn get_queen_castling_right_index(side: Side) -> usize {
        match side {
            Side::White => 8,
            Side::Black => 10,
        }
    }

    fn get_king_castling_right_index(side: Side) -> usize {
        match side {
            Side::White => 9,
            Side::Black => 11,
        }
    }

    fn get_en_passant_index(file: File) -> usize {
        file.index() as usize
    }

    pub fn get_queen_castling_right(&self, side: Side) -> bool {
        self.get_bit(Self::get_queen_castling_right_index(side))
    }

    pub fn set_queen_castling_right(&mut self, side: Side, castled: bool) -> () {
        self.set_bit(Self::get_queen_castling_right_index(side), castled)
    }

    pub fn get_king_castling_right(&self, side: Side) -> bool {
        self.get_bit(Self::get_king_castling_right_index(side))
    }

    pub fn set_king_castling_right(&mut self, side: Side, value: bool) -> () {
        self.set_bit(Self::get_king_castling_right_index(side), value)
    }

    pub fn get_en_passant_open(&self, file: File) -> bool {
        self.get_bit(Self::get_en_passant_index(file))
    }

    pub fn set_en_passant_open(&mut self, file: File, value: bool) -> () {
        self.set_bit(Self::get_en_passant_index(file), value)
    }

    pub fn get_whites_turn(&self) -> bool {
        self.get_bit(Self::WHITES_TURN_INDEX)
    }

    pub fn set_whites_turn(&mut self, value: bool) -> () {
        self.set_bit(Self::WHITES_TURN_INDEX, value)
    }
}

#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(
    feature = "std",
    derive(Clone, Debug, PartialEq, Eq, scale_info::TypeInfo, StorageLayout)
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
    flags: Flags,
}

impl Board {
    pub fn new(pieces: Vec<(Side, Piece, Square)>, flags: Flags) -> Self {
        let mut black = BitBoard::empty();
        let mut white = BitBoard::empty();
        let mut kings = BitBoard::empty();
        let mut queens = BitBoard::empty();
        let mut rooks = BitBoard::empty();
        let mut bishops = BitBoard::empty();
        let mut knights = BitBoard::empty();
        let mut pawns = BitBoard::empty();

        for (side, piece, square) in pieces.iter() {
            let bitboard = BitBoard::square(square.index());

            match side {
                Side::White => white |= bitboard,
                Side::Black => black |= bitboard,
            };

            match piece {
                Piece::Pawn => pawns |= bitboard,
                Piece::Knight => knights |= bitboard,
                Piece::Bishop => bishops |= bitboard,
                Piece::Rook => rooks |= bitboard,
                Piece::Queen => queens |= bitboard,
                Piece::King => kings |= bitboard,
            };
        }

        Self {
            black,
            white,
            kings,
            queens,
            rooks,
            bishops,
            knights,
            pawns,
            flags,
        }
    }

    pub fn default() -> Self {
        Self {
            black: 0xffff000000000000.into(),
            white: 0xffff.into(),
            kings: 0x1000000000000010.into(),
            queens: 0x800000000000008.into(),
            rooks: 0x8100000000000081.into(),
            bishops: 0x2400000000000024.into(),
            knights: 0x4200000000000042.into(),
            pawns: 0xff00000000ff00.into(),
            flags: Flags::default(),
        }
    }

    pub fn make_move(&mut self, ply: Ply) -> Result<Vec<Event>, Error> {
        let (from_side, from_piece) = self
            .get_piece_at(ply.from().index())
            .ok_or(Error::InvalidArgument)?;

        if from_side as u8 != self.get_side_turn() as u8 {
            return Err(Error::InvalidArgument);
        }

        todo!()
    }

    pub fn get_pieces(&self) -> Vec<(Side, Piece, Square)> {
        let mut pieces = Vec::new();

        for square_index in SQUARE_INDEX_RANGE {
            if let Some((side, piece)) = self.get_piece_at(square_index) {
                let square = Square::from_index(square_index);

                pieces.push((side, piece, square));
            }
        }

        pieces
    }

    pub fn get_flags(&self) -> &Flags {
        &self.flags
    }
}

impl Board {
    fn occupied(&self) -> BitBoard {
        self.black | self.white
    }

    fn empty(&self) -> BitBoard {
        !self.occupied()
    }

    fn get_side_turn(&self) -> Side {
        match self.get_flags().get_whites_turn() {
            true => Side::White,
            false => Side::Black,
        }
    }

    fn get_ray_attacks(&self, square_index: SquareIndex, direction: Direction) -> BitBoard {
        let mut attacks = BitBoard::ray_mask(square_index, direction);
        let blocker = attacks & self.occupied();

        if blocker.not_empty() {
            let square = blocker.bit_scan(direction.negative());
            attacks ^= BitBoard::ray_mask(square, direction);
        }

        attacks
    }

    fn get_pseudo_legal_moves(&self) -> HashMap<(Side, Piece, Square), BitBoard> {
        let mut moves = HashMap::new();

        for square_index in SQUARE_INDEX_RANGE {
            if let Some((side, piece)) = self.get_piece_at(square_index) {
                let bitboard = match (side, piece) {
                    (side, Piece::Knight) => {
                        BitBoard::knight_attacks(square_index) & !self.get_pieces_by_side(side)
                    }
                    (side, Piece::King) => {
                        BitBoard::king_attacks(square_index) & !self.get_pieces_by_side(side)
                    }
                    (side, Piece::Bishop) => {
                        todo!()
                    }
                    (side, Piece::Rook) => {
                        todo!()
                    }
                    (side, Piece::Queen) => {
                        todo!()
                    }
                    (Side::White, Piece::Pawn) => {
                        todo!()
                    }
                    (Side::Black, Piece::Pawn) => {
                        todo!()
                    }
                };

                let square = Square::from_index(square_index);

                moves.insert((side, piece, square), bitboard);
            };
        }

        moves
    }

    fn get_pieces_by_side(&self, side: Side) -> BitBoard {
        match side {
            Side::White => self.white,
            Side::Black => self.black,
        }
    }

    fn get_piece_at(&self, square_index: SquareIndex) -> Option<(Side, Piece)> {
        let side = if self.white.get(square_index) {
            Side::White
        } else if self.black.get(square_index) {
            Side::Black
        } else {
            return None;
        };

        let piece = if self.pawns.get(square_index) {
            Piece::Pawn
        } else if self.knights.get(square_index) {
            Piece::Knight
        } else if self.bishops.get(square_index) {
            Piece::Bishop
        } else if self.rooks.get(square_index) {
            Piece::Rook
        } else if self.queens.get(square_index) {
            Piece::Queen
        } else {
            Piece::King
        };

        return Some((side, piece));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_pawn_positions() {
        let board = Board::default();

        assert_eq!(board.pawns, 0xff00000000ff00.into());
    }

    #[test]
    fn board_rook_positions() {
        let board = Board::default();

        assert_eq!(board.rooks, 0x8100000000000081.into());
    }

    #[test]
    fn board_knight_positions() {
        let board = Board::default();

        assert_eq!(board.knights, 0x4200000000000042.into());
    }

    #[test]
    fn board_bishop_positions() {
        let board = Board::default();

        assert_eq!(board.bishops, 0x2400000000000024.into());
    }

    #[test]
    fn board_queen_positions() {
        let board = Board::default();

        assert_eq!(board.queens, 0x800000000000008.into());
    }

    #[test]
    fn board_king_positions() {
        let board = Board::default();

        assert_eq!(board.kings, 0x1000000000000010.into());
    }

    #[test]
    fn board_black_positions() {
        let board = Board::default();

        assert_eq!(board.black, 0xffff000000000000.into());
    }

    #[test]
    fn board_white_positions() {
        let board = Board::default();

        assert_eq!(board.white, 0xffff.into());
    }

    #[test]
    fn ray_attacks_sw_g5() {
        let board = Board::default();
        let square = Square::new(File::G, Rank::_5);

        assert_eq!(
            board.get_ray_attacks(square.index(), Direction::SouthWest),
            0x20100800.into()
        );
    }

    #[test]
    fn ray_attacks_n_d5() {
        let board = Board::default();
        let square = Square::new(File::D, Rank::_5);

        assert_eq!(
            board.get_ray_attacks(square.index(), Direction::North),
            0x8080000000000.into()
        );
    }

    #[test]
    fn ray_attacks_sw_h8() {
        let pieces: Vec<(Side, Piece, Square)> =
            vec![(Side::White, Piece::Rook, Square::new(File::A, Rank::_1))]
                .into_iter()
                .collect();

        let board = Board::new(pieces, Flags::default());
        let square = Square::new(File::H, Rank::_8);

        assert_eq!(
            board.get_ray_attacks(square.index(), Direction::SouthWest),
            0x40201008040201.into()
        );
    }

    #[test]
    fn board_get_pieces() {
        let pieces: Vec<(Side, Piece, Square)> = vec![
            (Side::White, Piece::Rook, Square::from_index(0)),
            (Side::White, Piece::Knight, Square::from_index(1)),
            (Side::White, Piece::Bishop, Square::from_index(2)),
            (Side::White, Piece::Queen, Square::from_index(3)),
            (Side::White, Piece::King, Square::from_index(4)),
            (Side::White, Piece::Bishop, Square::from_index(5)),
            (Side::White, Piece::Knight, Square::from_index(6)),
            (Side::White, Piece::Rook, Square::from_index(7)),
            (Side::White, Piece::Pawn, Square::from_index(8)),
            (Side::White, Piece::Pawn, Square::from_index(9)),
            (Side::White, Piece::Pawn, Square::from_index(10)),
            (Side::White, Piece::Pawn, Square::from_index(11)),
            (Side::White, Piece::Pawn, Square::from_index(12)),
            (Side::White, Piece::Pawn, Square::from_index(13)),
            (Side::White, Piece::Pawn, Square::from_index(14)),
            (Side::White, Piece::Pawn, Square::from_index(15)),
            (Side::Black, Piece::Pawn, Square::from_index(48)),
            (Side::Black, Piece::Pawn, Square::from_index(49)),
            (Side::Black, Piece::Pawn, Square::from_index(50)),
            (Side::Black, Piece::Pawn, Square::from_index(51)),
            (Side::Black, Piece::Pawn, Square::from_index(52)),
            (Side::Black, Piece::Pawn, Square::from_index(53)),
            (Side::Black, Piece::Pawn, Square::from_index(54)),
            (Side::Black, Piece::Pawn, Square::from_index(55)),
            (Side::Black, Piece::Rook, Square::from_index(56)),
            (Side::Black, Piece::Knight, Square::from_index(57)),
            (Side::Black, Piece::Bishop, Square::from_index(58)),
            (Side::Black, Piece::Queen, Square::from_index(59)),
            (Side::Black, Piece::King, Square::from_index(60)),
            (Side::Black, Piece::Bishop, Square::from_index(61)),
            (Side::Black, Piece::Knight, Square::from_index(62)),
            (Side::Black, Piece::Rook, Square::from_index(63)),
        ]
        .into_iter()
        .collect();

        assert_eq!(Board::default().get_pieces(), pieces);
    }
}
