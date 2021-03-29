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
use crate::dot_chess::Error;
use bitintr::Tzcnt;
use core::convert::TryFrom;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use ink_storage::Vec;
use scale::{Decode, Encode};

pub use direction::Direction;
pub use event::Event;
pub use file::File;
pub use piece::Piece;
pub use ply::{Ply, Promotion};
pub use rank::Rank;
pub use side::Side;
pub use square::Square;

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

impl core::convert::Into<u16> for Flags {
    fn into(self) -> u16 {
        self.0
    }
}

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
    halfmove_clock: u32,
}

impl Board {
    pub fn new(pieces: Vec<(Side, Piece, Square)>, flags: Flags, halfmove_clock: u32) -> Self {
        let mut black = BitBoard::EMPTY;
        let mut white = BitBoard::EMPTY;
        let mut kings = BitBoard::EMPTY;
        let mut queens = BitBoard::EMPTY;
        let mut rooks = BitBoard::EMPTY;
        let mut bishops = BitBoard::EMPTY;
        let mut knights = BitBoard::EMPTY;
        let mut pawns = BitBoard::EMPTY;

        for (side, piece, square) in pieces.iter() {
            let bitboard: BitBoard = (*square).into();

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
            halfmove_clock,
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
            halfmove_clock: 0,
        }
    }

    pub fn halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }

    // TODO test
    pub fn try_make_move(&self, ply: Ply) -> Result<(Self, Vec<Event>), Error> {
        // Assert move is pseudo legal
        if (self.get_pseudo_legal_moves_from(ply.from()) & BitBoard::square(ply.to())).is_empty() {
            return Err(Error::IllegalMove);
        }

        let (board, events) = self.try_make_pseudo_legal_move(ply)?;

        // Assert king not attacked
        let side_moved = self.get_side_turn();
        let king_square = board.get_king_square(side_moved);
        if board.is_attacked(king_square, side_moved.flip()) {
            return Err(Error::IllegalMove);
        }

        Ok((board, events))
    }

    // TODO test
    pub fn side_has_legal_move(&self, side: Side) -> bool {
        let mut pieces = self.get_pieces_by_side(side);

        while pieces.not_empty() {
            let square = pieces.pop_square();

            if self.get_legal_moves_from(square).not_empty() {
                return true;
            }
        }

        false
    }

    // TODO test
    pub fn get_legal_moves_from(&self, from: Square) -> BitBoard {
        let mut moves = BitBoard::EMPTY;
        let mut move_bb = self.get_pseudo_legal_moves_from(from);

        while move_bb.not_empty() {
            let to = move_bb.pop_square();
            let ply = Ply::new(from, to, Promotion::QUEEN_PROMOTION); // Use queen promotion in case its a promo-move, otherwise it doesn't matter

            if let Ok((board, _)) = self.try_make_pseudo_legal_move(ply) {
                // Assert king not attacked
                let side_moved = self.get_side_turn();
                let king_square = board.get_king_square(side_moved);
                if board.is_attacked(king_square, side_moved.flip()) {
                    continue;
                }

                moves |= BitBoard::square(to);
            }
        }

        moves
    }

    pub fn get_pieces(&self) -> Vec<(Side, Piece, Square)> {
        let mut pieces = Vec::new();
        let mut occupied = self.occupied();

        while occupied.not_empty() {
            let square = occupied.pop_square();

            if let Some((side, piece)) = self.get_piece_at(square) {
                pieces.push((side, piece, square));
            }
        }

        pieces
    }

    pub fn get_king_square(&self, side: Side) -> Square {
        let pieces = match side {
            Side::White => self.white,
            Side::Black => self.black,
        };

        (pieces & self.kings).pop_square()
    }

    // TODO test
    pub fn is_attacked(&self, square: Square, by_side: Side) -> bool {
        let bitboard = BitBoard::square(square);
        let attack_pieces = match by_side {
            Side::White => self.white,
            Side::Black => self.black,
        };

        let pawns = attack_pieces & self.pawns;
        if (bitboard.black_pawn_any_attacks_mask() & pawns).not_empty() {
            return true;
        }

        let knights = attack_pieces & self.knights;
        if (BitBoard::knight_attacks_mask(square) & knights).not_empty() {
            return true;
        }

        let kings = attack_pieces & self.kings;
        if (BitBoard::king_attacks_mask(square) & kings).not_empty() {
            return true;
        }

        let bishops_queens = attack_pieces & (self.bishops | self.queens);
        if (self.bishop_attacks(square) & bishops_queens).not_empty() {
            return true;
        }

        let rooks_queens = attack_pieces & (self.rooks | self.queens);
        if (self.rook_attacks(square) & rooks_queens).not_empty() {
            return true;
        }

        false
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

    fn duplicate(&self) -> Self {
        Self {
            black: self.black,
            white: self.white,
            kings: self.kings,
            queens: self.queens,
            rooks: self.rooks,
            bishops: self.bishops,
            knights: self.knights,
            pawns: self.pawns,
            flags: Flags(self.flags.0),
            halfmove_clock: self.halfmove_clock,
        }
    }

    fn get_ray_attacks(&self, square: Square, direction: Direction) -> BitBoard {
        let mut attacks = BitBoard::ray_mask(square, direction);
        let blocker = attacks & self.occupied();

        if blocker.not_empty() {
            let square = if direction.negative() {
                blocker.bit_scan_reverse()
            } else {
                blocker.bit_scan_forward()
            };

            attacks ^= BitBoard::ray_mask(square, direction);
        }

        attacks
    }

    fn diagonal_attacks(&self, square: Square) -> BitBoard {
        self.get_ray_attacks(square, Direction::NorthEast)
            | self.get_ray_attacks(square, Direction::SouthWest)
    }

    fn anti_diagonal_attacks(&self, square: Square) -> BitBoard {
        self.get_ray_attacks(square, Direction::NorthWest)
            | self.get_ray_attacks(square, Direction::SouthEast)
    }

    fn file_attacks(&self, square: Square) -> BitBoard {
        self.get_ray_attacks(square, Direction::North)
            | self.get_ray_attacks(square, Direction::South)
    }

    fn rank_attacks(&self, square: Square) -> BitBoard {
        self.get_ray_attacks(square, Direction::East)
            | self.get_ray_attacks(square, Direction::West)
    }

    fn rook_attacks(&self, square: Square) -> BitBoard {
        self.file_attacks(square) | self.rank_attacks(square)
    }

    fn bishop_attacks(&self, square: Square) -> BitBoard {
        self.diagonal_attacks(square) | self.anti_diagonal_attacks(square)
    }

    fn queen_attacks(&self, square: Square) -> BitBoard {
        self.rook_attacks(square) | self.bishop_attacks(square)
    }

    // TODO test
    fn try_make_pseudo_legal_move(&self, ply: Ply) -> Result<(Self, Vec<Event>), Error> {
        // Assert sides turn
        let (side, piece) = self
            .get_piece_at(ply.from())
            .ok_or(Error::InvalidArgument)?;

        if side as u8 != self.get_side_turn() as u8 {
            return Err(Error::InvalidArgument);
        }

        // Make new board and event bag
        let mut board_new = self.duplicate();
        let mut events = Vec::new();

        // Reset en passants
        for file in board_new.flags.get_en_passant_open_files().iter() {
            events.push(Event::EnPassantClosed(*file));
        }

        let from = ply.from();
        let to = ply.to();

        board_new.flags.reset_en_passant_open_files();

        let opponent_side = side.flip();
        let opponent_pieces = self.get_pieces_by_side(opponent_side);

        // Move & capture pieces
        match piece {
            Piece::Pawn => {
                let file_from: File = from.into();
                let file_to: File = to.into();
                let rank_to: Rank = to.into();

                // Reset halfmove clock
                board_new.halfmove_clock = 0;

                // Is capture?
                if file_from != file_to {
                    let en_passant = (BitBoard::square(to) & opponent_pieces).is_empty();
                    let captured_square = if en_passant {
                        match side {
                            Side::White => BitBoard::from(to).south_one().pop_square(),
                            Side::Black => BitBoard::from(to).north_one().pop_square(),
                        }
                    } else {
                        to
                    };

                    let (_, captured_piece) = board_new.get_piece_at(captured_square).unwrap();

                    board_new.clear_piece(captured_square);

                    events.push(Event::PieceLeftSquare(
                        opponent_side,
                        captured_piece,
                        captured_square,
                    ));

                    board_new.clear_piece(from);
                    board_new.set_piece(side, piece, to);

                    events.push(Event::PieceLeftSquare(side, piece, from));
                    events.push(Event::PieceEnteredSquare(side, piece, to));
                } else {
                    // Is double push?
                    let rank_from: Rank = from.into();

                    if let (Rank::_2, Rank::_4) | (Rank::_7, Rank::_5) = (rank_from, rank_to) {
                        let file: File = to.into();
                        board_new.flags.set_en_passant_open(file, true);
                        events.push(Event::EnPassantOpened(file));

                        board_new.clear_piece(from);
                        board_new.set_piece(side, piece, to);

                        events.push(Event::PieceLeftSquare(side, piece, from));
                        events.push(Event::PieceEnteredSquare(side, piece, to));

                        return Ok((board_new, events));
                    }
                }

                // Is promotion?
                if let Rank::_8 | Rank::_1 = rank_to {
                    let new_piece = ply.promotion().ok_or(Error::InvalidArgument)?;

                    board_new.clear_piece(from);
                    board_new.set_piece(side, new_piece, to);

                    events.push(Event::PieceLeftSquare(side, piece, from));
                    events.push(Event::PieceEnteredSquare(side, new_piece, to));
                } else {
                    board_new.clear_piece(from);
                    board_new.set_piece(side, piece, to);

                    events.push(Event::PieceLeftSquare(side, piece, from));
                    events.push(Event::PieceEnteredSquare(side, piece, to));
                }
            }
            Piece::King => {
                // Handle captures
                let is_capture = (BitBoard::square(to) & opponent_pieces).not_empty();
                if is_capture {
                    let (_, captured_piece) = board_new.get_piece_at(to).unwrap();
                    board_new.clear_piece(to);
                    events.push(Event::PieceLeftSquare(opponent_side, captured_piece, to));

                    // Reset halfmove clock
                    board_new.halfmove_clock = 0;
                } else {
                    let (
                        king_square,
                        cstl_ks_k_square,
                        cstl_ks_r_from_square,
                        cstl_ks_r_to_square,
                        cstl_qs_k_square,
                        cstl_qs_r_from_square,
                        cstl_qs_r_to_square,
                    ) = match side {
                        Side::White => (
                            Square::E1,
                            Square::G1,
                            Square::H1,
                            Square::F1,
                            Square::C1,
                            Square::A1,
                            Square::D1,
                        ),
                        Side::Black => (
                            Square::E8,
                            Square::G8,
                            Square::H8,
                            Square::F8,
                            Square::C8,
                            Square::A8,
                            Square::D8,
                        ),
                    };

                    // Is castling?
                    if from == king_square {
                        match to {
                            to if to == cstl_ks_k_square => {
                                board_new.clear_piece(cstl_ks_r_from_square);
                                board_new.set_piece(side, Piece::Rook, cstl_ks_r_to_square);

                                events.push(Event::PieceLeftSquare(
                                    side,
                                    Piece::Rook,
                                    cstl_ks_r_from_square,
                                ));
                                events.push(Event::PieceEnteredSquare(
                                    side,
                                    Piece::Rook,
                                    cstl_ks_r_to_square,
                                ));
                            }
                            to if to == cstl_qs_k_square => {
                                board_new.clear_piece(cstl_qs_r_from_square);
                                board_new.set_piece(side, Piece::Rook, cstl_qs_r_to_square);

                                events.push(Event::PieceLeftSquare(
                                    side,
                                    Piece::Rook,
                                    cstl_qs_r_from_square,
                                ));
                                events.push(Event::PieceEnteredSquare(
                                    side,
                                    Piece::Rook,
                                    cstl_qs_r_to_square,
                                ));
                            }
                            _ => {}
                        }
                    }
                }

                // Revoke castling rights if not already
                if self.get_flags().get_king_side_castling_right(side) {
                    board_new.flags.set_king_side_castling_right(side, false);
                    events.push(Event::KingSideCastlingRightLost(side));
                }

                if self.get_flags().get_queen_side_castling_right(side) {
                    board_new.flags.set_queen_side_castling_right(side, false);
                    events.push(Event::QueenSideCastlingRightLost(side));
                }

                board_new.clear_piece(from);
                board_new.set_piece(side, piece, to);

                events.push(Event::PieceLeftSquare(side, piece, from));
                events.push(Event::PieceEnteredSquare(side, piece, to));
            }
            Piece::Knight | Piece::Bishop | Piece::Queen => {
                // Handle captures
                let is_capture = (BitBoard::square(to) & opponent_pieces).not_empty();
                if is_capture {
                    let (_, captured_piece) = board_new.get_piece_at(to).unwrap();
                    board_new.clear_piece(to);
                    events.push(Event::PieceLeftSquare(opponent_side, captured_piece, to));

                    // Reset halfmove clock
                    board_new.halfmove_clock = 0;
                }

                board_new.clear_piece(from);
                board_new.set_piece(side, piece, to);

                events.push(Event::PieceLeftSquare(side, piece, from));
                events.push(Event::PieceEnteredSquare(side, piece, to));
            }
            Piece::Rook => {
                // Handle captures
                let is_capture = (BitBoard::square(to) & opponent_pieces).not_empty();
                if is_capture {
                    let (_, captured_piece) = board_new.get_piece_at(to).unwrap();
                    board_new.clear_piece(to);
                    events.push(Event::PieceLeftSquare(opponent_side, captured_piece, to));

                    // Reset halfmove clock
                    board_new.halfmove_clock = 0;
                }

                // Revoke castling rights
                let (king_side_origin, queen_side_origin) = match side {
                    Side::White => (Square::H1, Square::A1),
                    Side::Black => (Square::H8, Square::A8),
                };

                if from == king_side_origin && self.get_flags().get_king_side_castling_right(side) {
                    board_new.flags.set_king_side_castling_right(side, false);
                    events.push(Event::KingSideCastlingRightLost(side));
                } else if from == queen_side_origin
                    && self.get_flags().get_queen_side_castling_right(side)
                {
                    board_new.flags.set_queen_side_castling_right(side, false);
                    events.push(Event::QueenSideCastlingRightLost(side));
                }

                board_new.clear_piece(from);
                board_new.set_piece(side, piece, to);

                events.push(Event::PieceLeftSquare(side, piece, from));
                events.push(Event::PieceEnteredSquare(side, piece, to));
            }
        }

        events.push(Event::NextTurn(opponent_side));

        Ok((board_new, events))
    }

    // TODO test
    fn get_pseudo_legal_moves_from(&self, from: Square) -> BitBoard {
        match self.get_piece_at(from) {
            None => BitBoard::EMPTY,
            Some((side, piece)) => {
                let not_own_pieces = !self.get_pieces_by_side(side);

                match (side, piece) {
                    (_, Piece::Bishop) => self.bishop_attacks(from) & not_own_pieces,
                    (_, Piece::Rook) => self.rook_attacks(from) & not_own_pieces,
                    (_, Piece::Queen) => self.queen_attacks(from) & not_own_pieces,
                    (_, Piece::Knight) => BitBoard::knight_attacks_mask(from) & not_own_pieces,
                    (_, Piece::King) => {
                        let not_occuppied = !self.occupied();
                        let king = BitBoard::square(from);

                        let mut castling_queen_side = BitBoard::EMPTY;

                        if self.get_flags().get_queen_side_castling_right(side) {
                            castling_queen_side |= (king.west_one() & not_occuppied).west_one()
                                & not_occuppied
                                & BitBoard::FILE_C;
                        }

                        let mut castling_king_side = BitBoard::EMPTY;

                        if self.get_flags().get_king_side_castling_right(side) {
                            castling_king_side |= (king.east_one() & not_occuppied).east_one()
                                & not_occuppied
                                & BitBoard::FILE_G;
                        }

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

                        ((any_attacks & (pas_targets | white_pawns)) | any_targets) & not_own_pieces
                    }
                }
            }
        }
    }

    fn set_piece(&mut self, side: Side, piece: Piece, square: Square) {
        let bb = BitBoard::square(square);

        match side {
            Side::White => self.white |= bb,
            Side::Black => self.black |= bb,
        }

        match piece {
            Piece::Pawn => self.pawns |= bb,
            Piece::Knight => self.knights |= bb,
            Piece::Bishop => self.bishops |= bb,
            Piece::Rook => self.rooks |= bb,
            Piece::Queen => self.queens |= bb,
            Piece::King => self.kings |= bb,
        }
    }

    fn clear_piece(&mut self, square: Square) {
        let not_bb = !BitBoard::square(square);

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

    fn get_piece_at(&self, square: Square) -> Option<(Side, Piece)> {
        let side = if self.white.get(square) {
            Side::White
        } else if self.black.get(square) {
            Side::Black
        } else {
            return None;
        };

        let piece = if self.pawns.get(square) {
            Piece::Pawn
        } else if self.knights.get(square) {
            Piece::Knight
        } else if self.bishops.get(square) {
            Piece::Bishop
        } else if self.rooks.get(square) {
            Piece::Rook
        } else if self.queens.get(square) {
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
            board.get_ray_attacks(square, Direction::SouthWest),
            0x20100800.into()
        );
    }

    #[test]
    fn ray_attacks_n_d5() {
        let board = Board::default();
        let square = Square::new(File::D, Rank::_5);

        assert_eq!(
            board.get_ray_attacks(square, Direction::North),
            0x8080000000000.into()
        );
    }

    #[test]
    fn ray_attacks_sw_h8() {
        let pieces: Vec<(Side, Piece, Square)> =
            vec![(Side::White, Piece::Rook, Square::new(File::A, Rank::_1))]
                .into_iter()
                .collect();

        let board = Board::new(pieces, Flags::default(), 0);
        let square = Square::new(File::H, Rank::_8);

        assert_eq!(
            board.get_ray_attacks(square, Direction::SouthWest),
            0x40201008040201.into()
        );
    }

    #[test]
    fn board_get_pieces() {
        let pieces: Vec<(Side, Piece, Square)> = vec![
            (Side::White, Piece::Rook, 0u8.into()),
            (Side::White, Piece::Knight, 1u8.into()),
            (Side::White, Piece::Bishop, 2u8.into()),
            (Side::White, Piece::Queen, 3u8.into()),
            (Side::White, Piece::King, 4u8.into()),
            (Side::White, Piece::Bishop, 5u8.into()),
            (Side::White, Piece::Knight, 6u8.into()),
            (Side::White, Piece::Rook, 7u8.into()),
            (Side::White, Piece::Pawn, 8u8.into()),
            (Side::White, Piece::Pawn, 9u8.into()),
            (Side::White, Piece::Pawn, 10u8.into()),
            (Side::White, Piece::Pawn, 11u8.into()),
            (Side::White, Piece::Pawn, 12u8.into()),
            (Side::White, Piece::Pawn, 13u8.into()),
            (Side::White, Piece::Pawn, 14u8.into()),
            (Side::White, Piece::Pawn, 15u8.into()),
            (Side::Black, Piece::Pawn, 48u8.into()),
            (Side::Black, Piece::Pawn, 49u8.into()),
            (Side::Black, Piece::Pawn, 50u8.into()),
            (Side::Black, Piece::Pawn, 51u8.into()),
            (Side::Black, Piece::Pawn, 52u8.into()),
            (Side::Black, Piece::Pawn, 53u8.into()),
            (Side::Black, Piece::Pawn, 54u8.into()),
            (Side::Black, Piece::Pawn, 55u8.into()),
            (Side::Black, Piece::Rook, 56u8.into()),
            (Side::Black, Piece::Knight, 57u8.into()),
            (Side::Black, Piece::Bishop, 58u8.into()),
            (Side::Black, Piece::Queen, 59u8.into()),
            (Side::Black, Piece::King, 60u8.into()),
            (Side::Black, Piece::Bishop, 61u8.into()),
            (Side::Black, Piece::Knight, 62u8.into()),
            (Side::Black, Piece::Rook, 63u8.into()),
        ]
        .into_iter()
        .collect();

        assert_eq!(Board::default().get_pieces(), pieces);
    }
}
