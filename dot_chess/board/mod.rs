mod bitboard;
mod direction;
mod event;
mod file;
mod piece;
mod ply;
mod rank;
mod side;
mod square;

use std::convert::TryFrom;

use self::bitboard::BitBoard;
use self::square::SQUARE_INDEX_RANGE;
use crate::dot_chess::Error;
use bitintr::Tzcnt;
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
/// 1 << 8  White queen side castling right
/// 1 << 9  White king side castling right
/// 1 << 10 Black queen side castling right
/// 1 << 11 Black king side castling right
/// 1 << 12 Whites turn
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

    fn get_queen_side_castling_right_index(side: Side) -> usize {
        match side {
            Side::White => 8,
            Side::Black => 10,
        }
    }

    fn get_king_side_castling_right_index(side: Side) -> usize {
        match side {
            Side::White => 9,
            Side::Black => 11,
        }
    }

    fn get_en_passant_index(file: File) -> usize {
        file.index() as usize
    }

    pub fn get_queen_side_castling_right(&self, side: Side) -> bool {
        self.get_bit(Self::get_queen_side_castling_right_index(side))
    }

    pub fn set_queen_side_castling_right(&mut self, side: Side, castled: bool) -> () {
        self.set_bit(Self::get_queen_side_castling_right_index(side), castled)
    }

    pub fn get_king_side_castling_right(&self, side: Side) -> bool {
        self.get_bit(Self::get_king_side_castling_right_index(side))
    }

    pub fn set_king_side_castling_right(&mut self, side: Side, value: bool) -> () {
        self.set_bit(Self::get_king_side_castling_right_index(side), value)
    }

    pub fn get_en_passant_open_files(&self) -> Vec<File> {
        let mut files = Vec::new();
        let mut mask = (self.0 & 0xffu16) as u8;
        let mut next = 0u8;

        while mask != 0 {
            let zcnt = mask.tzcnt();
            next += zcnt;
            mask ^= 1 << zcnt;
            files.push(File::try_from(next).unwrap());
        }

        files
    }

    pub fn reset_en_passant_open_files(&mut self) -> () {
        self.0 &= 0xff00u16;
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
        let mut black = BitBoard::EMPTY;
        let mut white = BitBoard::EMPTY;
        let mut kings = BitBoard::EMPTY;
        let mut queens = BitBoard::EMPTY;
        let mut rooks = BitBoard::EMPTY;
        let mut bishops = BitBoard::EMPTY;
        let mut knights = BitBoard::EMPTY;
        let mut pawns = BitBoard::EMPTY;

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

    // TODO test
    pub fn try_make_move(&self, ply: Ply) -> Result<(Self, Vec<Event>), Error> {
        // Assert move is pseudo legal
        if (self.get_pseudo_legal_moves(ply.from()) & BitBoard::square(ply.to())).is_empty() {
            return Err(Error::IllegalMove);
        }

        let (board, events) = self.try_make_pseudo_legal_move(ply)?;

        // Assert king not attacked
        let side_moved = self.get_side_turn();
        let king_square = board.get_king_square(side_moved);
        if board.attackers_to(king_square).not_empty() {
            return Err(Error::IllegalMove);
        }

        Ok((board, events))
    }

    // TODO test
    pub fn get_legal_moves(&self, from: SquareIndex) -> BitBoard {
        let mut moves = BitBoard::EMPTY;
        let mut move_bb = self.get_pseudo_legal_moves(from);

        while move_bb.not_empty() {
            let to = move_bb.pop_square();
            let ply = Ply::new(from, to, PlyFlags::DEFAULT);

            if let Ok((board, events)) = self.try_make_pseudo_legal_move(ply) {
                // Assert king not attacked
                let side_moved = self.get_side_turn();
                let king_square = board.get_king_square(side_moved);
                if board.attackers_to(king_square).not_empty() {
                    continue;
                }

                moves |= BitBoard::square(to);
            }
        }

        moves
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

    pub fn get_side_turn(&self) -> Side {
        match self.get_flags().get_whites_turn() {
            true => Side::White,
            false => Side::Black,
        }
    }
}

impl Board {
    // TODO make private (zobrist is dependent)
    pub fn get_flags(&self) -> &Flags {
        &self.flags
    }

    fn occupied(&self) -> BitBoard {
        self.black | self.white
    }

    fn empty(&self) -> BitBoard {
        !self.occupied()
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

    fn diagonal_attacks(&self, square_index: SquareIndex) -> BitBoard {
        self.get_ray_attacks(square_index, Direction::NorthEast)
            | self.get_ray_attacks(square_index, Direction::SouthWest)
    }

    fn anti_diagonal_attacks(&self, square_index: SquareIndex) -> BitBoard {
        self.get_ray_attacks(square_index, Direction::NorthWest)
            | self.get_ray_attacks(square_index, Direction::SouthEast)
    }

    fn file_attacks(&self, square_index: SquareIndex) -> BitBoard {
        self.get_ray_attacks(square_index, Direction::North)
            | self.get_ray_attacks(square_index, Direction::South)
    }

    fn rank_attacks(&self, square_index: SquareIndex) -> BitBoard {
        self.get_ray_attacks(square_index, Direction::East)
            | self.get_ray_attacks(square_index, Direction::West)
    }

    fn rook_attacks(&self, square_index: SquareIndex) -> BitBoard {
        self.file_attacks(square_index) | self.rank_attacks(square_index)
    }

    fn bishop_attacks(&self, square_index: SquareIndex) -> BitBoard {
        self.diagonal_attacks(square_index) | self.anti_diagonal_attacks(square_index)
    }

    fn queen_attacks(&self, square_index: SquareIndex) -> BitBoard {
        self.rook_attacks(square_index) | self.bishop_attacks(square_index)
    }

    fn get_king_square(&self, side: Side) -> SquareIndex {
        todo!()
    }

    fn attackers_to(&self, square_index: SquareIndex) -> BitBoard {
        todo!()
    }

    fn try_make_pseudo_legal_move(&self, ply: Ply) -> Result<(Self, Vec<Event>), Error> {
        // Assert sides turn
        let (side, piece) = self
            .get_piece_at(ply.from())
            .ok_or(Error::InvalidArgument)?;

        if side as u8 != self.get_side_turn() as u8 {
            return Err(Error::InvalidArgument);
        }

        // Make new board and event bag
        let mut board_new = self.clone();
        let mut events = Vec::new();

        // Reset en passants
        for file in board_new.flags.get_en_passant_open_files() {
            events.push(Event::EnPassantClosed(file));
        }

        board_new.flags.reset_en_passant_open_files();

        let opponent_side = side.flip();

        // Move & capture pieces
        match piece {
            Piece::Pawn => {
                let from_file = File::try_from(ply.from())?;
                let to_file = File::try_from(ply.to())?;

                let opponent_pieces = self.get_pieces_by_side(opponent_side);

                // Is capture?
                if from_file != to_file {
                    let en_passant = (BitBoard::square(ply.to()) & opponent_pieces).is_empty();
                    let captured_square = if en_passant {
                        match side {
                            Side::White => BitBoard::from(ply.to()).south_one().pop_square(),
                            Side::Black => BitBoard::from(ply.to()).north_one().pop_square(),
                        }
                    } else {
                        ply.to()
                    };

                    let (captured_side, captured_piece) =
                        board_new.get_piece_at(captured_square).unwrap();

                    board_new.clear_piece(captured_square);

                    events.push(Event::PieceLeftSquare(
                        captured_side,
                        captured_piece,
                        captured_square,
                    ));
                } else {
                    // Is double push?
                    let rank_from = Rank::from(ply.from());
                    let rank_to = Rank::from(ply.to());

                    if let (Rank::_2, Rank::_4) | (Rank::_7, Rank::_5) = (rank_from, rank_to) {
                        let file = File::from(ply.to());
                        board_new.flags.set_en_passant_open(file, true);
                        events.push(Event::EnPassantOpened(file));
                    }
                }

                board_new.clear_piece(ply.from());
                board_new.set_piece(side, piece, ply.to());

                events.push(Event::PieceLeftSquare(side, piece, ply.from()));
                events.push(Event::PieceEnteredSquare(side, piece, ply.to()));
            }
            Piece::Knight => {
                todo!()
            }
            Piece::Bishop => {
                todo!()
            }
            Piece::Rook => {
                todo!()
            }
            Piece::Queen => {
                todo!()
            }
            Piece::King => {
                todo!()
            }
        }

        events.push(Event::NextTurn(opponent_side));

        Ok((board_new, events))
    }

    // TODO test
    fn get_pseudo_legal_moves(&self, from: SquareIndex) -> BitBoard {
        match self.get_piece_at(from) {
            None => BitBoard::EMPTY,
            Some((side, piece)) => {
                let not_own_pieces = !self.get_pieces_by_side(side);

                match (side, piece) {
                    (side, Piece::Bishop) => self.bishop_attacks(from) & not_own_pieces,
                    (side, Piece::Rook) => self.rook_attacks(from) & not_own_pieces,
                    (side, Piece::Queen) => self.queen_attacks(from) & not_own_pieces,
                    (side, Piece::Knight) => BitBoard::knight_attacks_mask(from) & not_own_pieces,
                    (side, Piece::King) => {
                        let not_occuppied = !self.occupied();
                        let king = BitBoard::square(from);

                        let castling_queen_side = (king.west_one() & not_occuppied).west_one()
                            & not_occuppied
                            & BitBoard::FILE_C
                            & self.get_flags().get_queen_side_castling_right(side);

                        let castling_king_side = (king.east_one() & not_occuppied).east_one()
                            & not_occuppied
                            & BitBoard::FILE_G
                            & self.get_flags().get_king_side_castling_right(side);

                        (BitBoard::king_attacks_mask(from) & not_own_pieces)
                            | castling_king_side
                            | castling_queen_side
                    }
                    (Side::White, Piece::Pawn) => {
                        let black_pawns: BitBoard = self.black & self.pawns;
                        let white_pawns: BitBoard = self.white & self.pawns;
                        let any_attacks: BitBoard = white_pawns.white_pawn_any_attacks_mask();
                        let sgl_targets: BitBoard = white_pawns.north_one() & not_own_pieces;
                        let any_targets: BitBoard =
                            sgl_targets | sgl_targets.north_one() & BitBoard::RANK_4;

                        let pas_targets: BitBoard = BitBoard::RANK_6
                            & self
                                .get_flags()
                                .get_en_passant_open_files()
                                .iter()
                                .fold(BitBoard::EMPTY, |bb, file| bb & BitBoard::from(*file));

                        ((any_attacks & (pas_targets | black_pawns)) | any_targets) & not_own_pieces
                    }
                    (Side::Black, Piece::Pawn) => {
                        let white_pawns: BitBoard = self.white & self.pawns;
                        let black_pawns: BitBoard = self.black & self.pawns;
                        let any_attacks: BitBoard = black_pawns.black_pawn_any_attacks_mask();
                        let sgl_targets: BitBoard = black_pawns.south_one() & not_own_pieces;
                        let any_targets: BitBoard =
                            sgl_targets | sgl_targets.south_one() & BitBoard::RANK_5;

                        let pas_targets: BitBoard = BitBoard::RANK_3
                            & self
                                .get_flags()
                                .get_en_passant_open_files()
                                .iter()
                                .fold(BitBoard::EMPTY, |bb, file| bb & BitBoard::from(*file));

                        ((any_attacks & (pas_targets | black_pawns)) | any_targets) & not_own_pieces
                    }
                }
            }
        }
    }

    fn set_piece(&mut self, side: Side, piece: Piece, square_index: SquareIndex) {
        let bb = BitBoard::square(square_index);
        let not_bb = !bb;

        match side {
            Side::White => {
                self.black &= not_bb;
                self.white |= bb;
            }
            Side::Black => {
                self.white &= not_bb;
                self.black |= bb;
            }
        }

        match piece {
            Piece::Pawn => {
                self.pawns |= bb;
                self.knights &= not_bb;
                self.bishops &= not_bb;
                self.rooks &= not_bb;
                self.queens &= not_bb;
                self.kings &= not_bb;
            }
            Piece::Knight => {
                self.pawns &= not_bb;
                self.knights |= bb;
                self.bishops &= not_bb;
                self.rooks &= not_bb;
                self.queens &= not_bb;
                self.kings &= not_bb;
            }
            Piece::Bishop => {
                self.pawns &= not_bb;
                self.knights &= not_bb;
                self.bishops |= bb;
                self.rooks &= not_bb;
                self.queens &= not_bb;
                self.kings &= not_bb;
            }
            Piece::Rook => {
                self.pawns &= not_bb;
                self.knights &= not_bb;
                self.bishops &= not_bb;
                self.rooks |= bb;
                self.queens &= not_bb;
                self.kings &= not_bb;
            }
            Piece::Queen => {
                self.pawns &= not_bb;
                self.knights &= not_bb;
                self.bishops &= not_bb;
                self.rooks &= not_bb;
                self.queens |= bb;
                self.kings &= not_bb;
            }
            Piece::King => {
                self.pawns &= not_bb;
                self.knights &= not_bb;
                self.bishops &= not_bb;
                self.rooks &= not_bb;
                self.queens &= not_bb;
                self.kings |= bb;
            }
        }
    }

    fn clear_piece(&mut self, square_index: SquareIndex) {
        let not_bb = !BitBoard::square(square_index);

        self.black &= not_bb;
        self.white &= not_bb;
        self.knights &= not_bb;
        self.bishops &= not_bb;
        self.rooks &= not_bb;
        self.queens &= not_bb;
        self.kings &= not_bb;
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
