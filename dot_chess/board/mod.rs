mod bitboard;
mod event;
mod file;
mod piece;
mod ply;
mod rank;
mod side;
mod square;

use crate::dot_chess::Error;
use bitboard::BitBoard;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use ink_storage::Vec;
use scale::{Decode, Encode};

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
            .get_piece(ply.from().index())
            .ok_or(Error::InvalidArgument)?;

        if from_side as u8 != self.get_side_turn() as u8 {
            return Err(Error::InvalidArgument);
        }

        todo!()
    }

    fn empty(&self) -> BitBoard {
        !(self.black | self.white)
    }

    fn get_side_turn(&self) -> Side {
        match self.get_flags().get_whites_turn() {
            true => Side::White,
            false => Side::Black,
        }
    }

    fn get_piece(&self, square_index: SquareIndex) -> Option<(Side, Piece)> {
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

    pub fn get_pieces(&self) -> Vec<(Side, Piece, Square)> {
        let mut pieces = Vec::new();

        for square_index in 0..64 {
            if let Some((side, piece)) = self.get_piece(square_index) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_h8_on_bitboard() {
        let square = Square::new(File::H, Rank::_8);

        assert_eq!(
            BitBoard::square(square.index()),
            0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000.into()
        );
    }

    #[test]
    fn square_a1_on_bitboard() {
        let square = Square::new(File::A, Rank::_1);

        assert_eq!(
            BitBoard::square(square.index()),
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001.into()
        );
    }

    #[test]
    fn square_a2_on_bitboard() {
        let square = Square::new(File::A, Rank::_2);

        assert_eq!(
            BitBoard::square(square.index()),
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000001_00000000.into()
        );
    }

    #[test]
    fn square_b2_on_bitboard() {
        let square = Square::new(File::B, Rank::_2);

        assert_eq!(
            BitBoard::square(square.index()),
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000010_00000000.into()
        );
    }

    #[test]
    fn board_pawn_positions() {
        let board = Board::default();

        assert_eq!(
            board.pawns,
            0b00000000_11111111_00000000_00000000_00000000_00000000_11111111_00000000.into()
        );
    }

    #[test]
    fn board_rook_positions() {
        let board = Board::default();

        assert_eq!(
            board.rooks,
            0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_10000001.into()
        );
    }

    #[test]
    fn board_knight_positions() {
        let board = Board::default();

        assert_eq!(
            board.knights,
            0b01000010_00000000_00000000_00000000_00000000_00000000_00000000_01000010.into()
        );
    }

    #[test]
    fn board_bishop_positions() {
        let board = Board::default();

        assert_eq!(
            board.bishops,
            0b00100100_00000000_00000000_00000000_00000000_00000000_00000000_00100100.into()
        );
    }

    #[test]
    fn board_queen_positions() {
        let board = Board::default();

        assert_eq!(
            board.queens,
            0b00001000_00000000_00000000_00000000_00000000_00000000_00000000_00001000.into()
        );
    }

    #[test]
    fn board_king_positions() {
        let board = Board::default();

        assert_eq!(
            board.kings,
            0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00010000.into()
        );
    }

    #[test]
    fn board_black_positions() {
        let board = Board::default();

        assert_eq!(
            board.black,
            0b11111111_11111111_00000000_00000000_00000000_00000000_00000000_00000000.into()
        );
    }

    #[test]
    fn board_white_positions() {
        let board = Board::default();

        assert_eq!(
            board.white,
            0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_11111111.into()
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
