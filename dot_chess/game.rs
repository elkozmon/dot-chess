use crate::board::{BitBoard, Board, File, Piece, Mov, Rank, Side, Square};
use crate::dot_chess::{Error, Result};
use crate::zobrist::ZobristHash;
use alloc::string::String;
use bitintr::Tzcnt;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::fmt::Write;
use ink_storage::collections::HashMap;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use ink_storage::Box;
use ink_storage::Vec;
use scale::{Decode, Encode};

pub type HalfmoveClock = u32;
pub type FullmoveNumber = u32;

/// Game state bit mask
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
pub struct State(u16);

impl core::convert::Into<u16> for State {
    fn into(self) -> u16 {
        self.0
    }
}

impl core::convert::Into<ZobristHash> for State {
    fn into(self) -> ZobristHash {
        let mut zhash = ZobristHash::zero();

        for side in [Side::White, Side::Black].iter() {
            let side = *side;

            if self.king_side_castling_right(side) {
                zhash.flip_king_castling_right(side);
            }

            if self.queen_side_castling_right(side) {
                zhash.flip_king_castling_right(side);
            }
        }

        for file in self.en_passant_open_files().iter() {
            zhash.flip_en_passant_file(*file);
        }

        if let Side::Black = self.next_turn_side() {
            zhash.flip_next_turn();
        }

        zhash
    }
}

impl State {
    const WHITES_TURN_INDEX: usize = 12;

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn queen_side_castling_right(&self, side: Side) -> bool {
        self.bit(Self::queen_side_castling_right_index(side))
    }

    pub fn set_queen_side_castling_right(&mut self, side: Side, value: bool) -> () {
        self.set_bit(Self::queen_side_castling_right_index(side), value)
    }

    pub fn king_side_castling_right(&self, side: Side) -> bool {
        self.bit(Self::king_side_castling_right_index(side))
    }

    pub fn set_king_side_castling_right(&mut self, side: Side, value: bool) -> () {
        self.set_bit(Self::king_side_castling_right_index(side), value)
    }

    pub fn en_passant_open_files(&self) -> Vec<File> {
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

    pub fn en_passant_open(&self, file: File) -> bool {
        self.bit(Self::en_passant_index(file))
    }

    pub fn set_en_passant_open(&mut self, file: File, value: bool) -> () {
        self.set_bit(Self::en_passant_index(file), value)
    }

    pub fn next_turn_side(&self) -> Side {
        match self.bit(Self::WHITES_TURN_INDEX) {
            true => Side::White,
            false => Side::Black,
        }
    }

    pub fn set_next_turn_side(&mut self, side: Side) -> () {
        let value = match side {
            Side::White => true,
            Side::Black => false,
        };

        self.set_bit(Self::WHITES_TURN_INDEX, value)
    }
}

impl State {
    fn bit(&self, bit: usize) -> bool {
        ((self.0 >> bit) & 1u16) == 1
    }

    fn set_bit(&mut self, bit: usize, to: bool) -> () {
        self.0 = (self.0 & !(1u16 << bit)) | ((to as u16) << bit);
    }

    fn queen_side_castling_right_index(side: Side) -> usize {
        match side {
            Side::White => 8,
            Side::Black => 10,
        }
    }

    fn king_side_castling_right_index(side: Side) -> usize {
        match side {
            Side::White => 9,
            Side::Black => 11,
        }
    }

    fn en_passant_index(file: File) -> usize {
        <File as Into<u8>>::into(file) as usize
    }
}

#[derive(Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, scale_info::TypeInfo, StorageLayout)
)]
pub struct Game {
    board: Board,
    state: State,
    history: Box<Vec<ZobristHash>>,
    halfmove_clock: HalfmoveClock,
    fullmove_number: FullmoveNumber,
}

impl Game {
    pub const FEN_NEW_GAME: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn new(fen: &str) -> Result<Self> {
        let mut board = Board::empty();
        let mut state = State::zero();
        let mut halfmove_clock = 0;
        let mut fullmove_number = 0;

        Self::apply_fen(
            fen,
            &mut board,
            &mut state,
            &mut halfmove_clock,
            &mut fullmove_number,
        )?;

        let board_zhash: ZobristHash = board.into();
        let state_zhash: ZobristHash = state.into();

        let mut history = Vec::new();
        history.push(board_zhash ^ state_zhash);

        Ok(Self {
            board,
            state,
            history: Box::new(history),
            halfmove_clock,
            fullmove_number,
        })
    }

    pub fn fen(&self) -> Result<String> {
        let mut fen = String::new();
        let mut index = 63;
        let mut skips: u8 = 0;

        let write_skips = |buf: &mut String, skips: u8| -> Result<()> {
            if skips > 0 {
                write!(buf, "{}", skips)?;
            }

            Ok(())
        };

        // Write positions
        loop {
            match self.board.piece_at(index.into()) {
                Some((side, piece, ..)) => {
                    let mut char: char = piece.into();

                    char = match side {
                        Side::White => char.to_ascii_uppercase(),
                        Side::Black => char.to_ascii_lowercase(),
                    };

                    write_skips(&mut fen, skips)?;
                    write!(&mut fen, "{}", char)?;
                }
                None => skips += 1,
            }

            if index % 8 == 0 {
                write_skips(&mut fen, skips)?;
                skips = 0;

                if index != 0 {
                    write!(&mut fen, "/")?;
                }
            }

            if index == 0 {
                break;
            }

            index -= 1;
        }

        // Write side turn
        let turn_char: char = self.next_turn_side().into();
        write!(&mut fen, " {}", turn_char.to_ascii_lowercase())?;

        // Write castling rights
        let mut any_castling_right = false;

        for side in [Side::White, Side::Black].iter() {
            if self.state.king_side_castling_right(*side) {
                let char = match side {
                    Side::White => 'K',
                    Side::Black => 'k',
                };

                write!(&mut fen, "{}", char)?;
                any_castling_right = true;
            }

            if self.state.queen_side_castling_right(*side) {
                let char = match side {
                    Side::White => 'Q',
                    Side::Black => 'q',
                };

                write!(&mut fen, "{}", char)?;
                any_castling_right = true;
            }
        }

        if !any_castling_right {
            write!(&mut fen, "-")?;
        }

        // Write en passants
        write!(&mut fen, " ")?;

        let mut any_en_passant = false;
        let rank_char = match self.next_turn_side() {
            Side::White => '6',
            Side::Black => '3',
        };

        for file in self.state.en_passant_open_files().iter() {
            any_en_passant = true;

            let file_char: char = (*file).into();
            write!(&mut fen, "{}{}", file_char.to_ascii_lowercase(), rank_char)?;
        }

        if !any_en_passant {
            write!(&mut fen, "-")?;
        }

        // Write halfmove clock and fullmove number
        write!(
            &mut fen,
            " {} {}",
            self.halfmove_clock, self.fullmove_number
        )?;

        Ok(fen)
    }

    pub fn halfmove_clock(&self) -> HalfmoveClock {
        self.halfmove_clock
    }

    pub fn next_turn_side(&self) -> Side {
        self.state.next_turn_side()
    }

    pub fn is_check(&self) -> bool {
        self.board.is_king_attacked(self.next_turn_side())
    }

    pub fn is_repetition(&self) -> bool {
        let occ = *self
            .history
            .iter()
            .fold(HashMap::<ZobristHash, u32>::new(), |mut map, zhash| {
                *map.entry(*zhash).or_insert(0) += 1;
                map
            })
            .into_iter()
            .max_by_key(|(_, v)| *v)
            .map(|(_, v)| v)
            .unwrap();

        occ >= 3
    }

    pub fn has_sufficient_mating_material(&self) -> bool {
        let mut white_score = 0;
        let mut black_score = 0;

        for (side, piece, _) in self.board.pieces().iter() {
            let ref_score = match side {
                Side::White => &mut white_score,
                Side::Black => &mut black_score,
            };

            match piece {
                Piece::Knight | Piece::Bishop => *ref_score += 1,
                Piece::Pawn | Piece::Rook | Piece::Queen => *ref_score += 2,
                Piece::King => {}
            }

            drop(ref_score);

            if white_score > 1 || black_score > 1 {
                return true;
            }
        }

        false
    }

    // TODO test
    pub fn has_legal_moves(&self) -> bool {
        let mut pieces = self.board.pieces_by_side(self.next_turn_side());

        while pieces.not_empty() {
            let square = pieces.pop_square();

            if self.legal_moves_from(square).not_empty() {
                return true;
            }
        }

        false
    }

    // TODO test
    pub fn legal_moves_from(&self, from: Square) -> BitBoard {
        let mut legal_moves = BitBoard::EMPTY;
        let mut pseudo_moves = self.pseudo_legal_moves_from(from);

        while pseudo_moves.not_empty() {
            let to = pseudo_moves.pop_square();

            // Use queen promotion in case its a promo-move, otherwise it doesn't matter
            let mov = Mov::new(from, to, Some(Piece::Queen));

            let (board, ..) = self.make_pseudo_legal_move(mov).unwrap();

            // Assert king not attacked
            if board.is_king_attacked(self.next_turn_side()) {
                continue;
            }

            legal_moves |= BitBoard::square(to);
        }

        legal_moves
    }

    // TODO test
    pub fn make_move(&self, mov: Mov) -> Result<Self> {
        // Assert move is pseudo legal
        if (self.pseudo_legal_moves_from(mov.from()) & BitBoard::square(mov.to())).is_empty() {
            return Err(Error::IllegalMove);
        }

        let (board, state, zhash, halfmove_clock, fullmove_number) =
            self.make_pseudo_legal_move(mov)?;

        // Assert king not attacked
        if board.is_king_attacked(self.next_turn_side()) {
            return Err(Error::IllegalMove);
        }

        // Create new hash history
        let mut history: Vec<ZobristHash> = if halfmove_clock > 0 {
            self.history.iter().map(|zh| *zh).collect()
        } else {
            Vec::new()
        };

        history.push(zhash);

        Ok(Self {
            board,
            state,
            history: Box::new(history),
            halfmove_clock,
            fullmove_number,
        })
    }
}

impl Game {
    fn apply_fen(
        fen: &str,
        board: &mut Board,
        state: &mut State,
        halfmove_clock: &mut HalfmoveClock,
        fullmove_number: &mut FullmoveNumber,
    ) -> Result<()> {
        let mut fen_chars = fen.chars();

        // Parse positions
        let mut index: u8 = 0;

        loop {
            let char = fen_chars.nth(0).ok_or(Error::InvalidArgument)?;

            if char.is_whitespace() {
                break;
            }

            if char.is_numeric() {
                index += char.to_digit(10).ok_or(Error::InvalidArgument)? as u8;
                continue;
            }

            if char.is_alphabetic() {
                let side = if char.is_uppercase() {
                    Side::White
                } else {
                    Side::Black
                };

                let piece: Piece = char.try_into()?;
                let square: Square = index.into();

                board.set_piece(side, piece, square);
                continue;
            }

            // Unexpected character
            return Err(Error::InvalidArgument);
        }

        // Validate entire board has been populated
        if index != 63 {
            return Err(Error::InvalidArgument);
        }

        // Parse turn
        let char = fen_chars.nth(0).ok_or(Error::InvalidArgument)?;

        let side = match char {
            'w' => Side::White,
            'b' => Side::Black,
            _ => return Err(Error::InvalidArgument),
        };

        state.set_next_turn_side(side);

        // Parse castling rights
        fen_chars.advance_by(1).or(Err(Error::InvalidArgument))?;

        loop {
            let char = fen_chars.nth(0).ok_or(Error::InvalidArgument)?;

            if char.is_whitespace() || char == '-' {
                break;
            }

            if char.is_alphabetic() {
                let side = if char.is_uppercase() {
                    Side::White
                } else {
                    Side::Black
                };

                match char.to_ascii_lowercase() {
                    'q' => state.set_queen_side_castling_right(side, true),
                    'k' => state.set_king_side_castling_right(side, true),
                    _ => return Err(Error::InvalidArgument),
                }

                continue;
            }

            // Unexpected character
            return Err(Error::InvalidArgument);
        }

        // Parse en passants
        fen_chars.advance_by(1).or(Err(Error::InvalidArgument))?;

        loop {
            let char = fen_chars.nth(0).ok_or(Error::InvalidArgument)?;

            if char.is_whitespace() || char == '-' {
                break;
            }

            if char.is_numeric() {
                continue;
            }

            if char.is_alphabetic() {
                let file: File = char.to_ascii_lowercase().try_into()?;
                state.set_en_passant_open(file, true);
                continue;
            }

            // Unexpected character
            return Err(Error::InvalidArgument);
        }

        // Parse halfmove clock
        *halfmove_clock = fen_chars
            .nth(1)
            .ok_or(Error::InvalidArgument)?
            .to_digit(10)
            .ok_or(Error::InvalidArgument)?;

        // Parse Fullmove number
        *fullmove_number = fen_chars
            .nth(1)
            .ok_or(Error::InvalidArgument)?
            .to_digit(10)
            .ok_or(Error::InvalidArgument)?;

        Ok(())
    }

    fn pseudo_legal_moves_from(&self, from: Square) -> BitBoard {
        let side = self.next_turn_side();

        self.board.pseudo_legal_moves_from(
            from,
            self.state.king_side_castling_right(side),
            self.state.queen_side_castling_right(side),
            self.state.en_passant_open_files(),
        )
    }

    // TODO test
    // TODO revoke opponents castling rights if this move attacks his castling path
    fn make_pseudo_legal_move(
        &self,
        mov: Mov,
    ) -> Result<(Board, State, ZobristHash, HalfmoveClock, FullmoveNumber)> {
        // Assert sides turn
        let (side, piece) = self
            .board
            .piece_at(mov.from())
            .ok_or(Error::InvalidArgument)?;

        if side as u8 != self.next_turn_side() as u8 {
            return Err(Error::InvalidArgument);
        }

        let from = mov.from();
        let to = mov.to();

        let opponent_side = side.flip();
        let opponent_pieces = self.board.pieces_by_side(opponent_side);

        // Make new board and event bag
        let mut board_new: Board = self.board.clone();
        let mut state_new: State = self.state.clone();
        let mut zhash_new: ZobristHash = *self.history.last().unwrap();
        let mut halfmove_clock_new = self.halfmove_clock + 1;
        let fullmove_number_new = match side {
            Side::White => self.fullmove_number,
            Side::Black => self.fullmove_number + 1,
        };

        // Reset en passants
        state_new.reset_en_passant_open_files();
        for file in self.state.en_passant_open_files().iter() {
            zhash_new.flip_en_passant_file(*file);
        }

        // Switch turns
        state_new.set_next_turn_side(opponent_side);
        zhash_new.flip_next_turn();

        // Move & capture pieces
        match piece {
            Piece::Pawn => {
                let file_from: File = from.into();
                let rank_from: Rank = from.into();
                let file_to: File = to.into();
                let rank_to: Rank = to.into();

                // Reset halfmove clock
                halfmove_clock_new = 0;

                // Is double push?
                if let (Rank::_2, Rank::_4) | (Rank::_7, Rank::_5) = (rank_from, rank_to) {
                    let file: File = to.into();

                    // Mark open en passant
                    state_new.set_en_passant_open(file, true);
                    zhash_new.flip_en_passant_file(file);

                    // Move our piece
                    board_new.clear_piece(from);
                    board_new.set_piece(side, piece, to);
                    zhash_new.flip_piece_position(side, piece, from);
                    zhash_new.flip_piece_position(side, piece, to);

                    // Early return
                    return Ok((
                        board_new,
                        state_new,
                        zhash_new,
                        halfmove_clock_new,
                        fullmove_number_new,
                    ));
                }

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

                    let (_, captured_piece) = board_new.piece_at(captured_square).unwrap();

                    // Capture opponents piece
                    board_new.clear_piece(captured_square);
                    zhash_new.flip_piece_position(opponent_side, captured_piece, captured_square);
                }

                // Is promotion?
                let new_piece = if let Rank::_8 | Rank::_1 = rank_to {
                    mov.promotion().ok_or(Error::InvalidArgument)?
                } else {
                    piece
                };

                // Move our piece
                board_new.clear_piece(from);
                board_new.set_piece(side, new_piece, to);
                zhash_new.flip_piece_position(side, piece, from);
                zhash_new.flip_piece_position(side, new_piece, to);
            }
            Piece::King => {
                // Revoke castling rights if not already
                if self.state.king_side_castling_right(side) {
                    state_new.set_king_side_castling_right(side, false);
                    zhash_new.flip_king_castling_right(side);
                }

                if self.state.queen_side_castling_right(side) {
                    state_new.set_queen_side_castling_right(side, false);
                    zhash_new.flip_queen_castling_right(side);
                }

                // Is capture?
                if (BitBoard::square(to) & opponent_pieces).not_empty() {
                    let (_, captured_piece) = board_new.piece_at(to).unwrap();

                    // Capture opponents piece
                    board_new.clear_piece(to);
                    zhash_new.flip_piece_position(opponent_side, captured_piece, to);

                    // Reset halfmove clock
                    halfmove_clock_new = 0;

                    // Move our piece
                    board_new.clear_piece(from);
                    board_new.set_piece(side, piece, to);
                    zhash_new.flip_piece_position(side, piece, from);
                    zhash_new.flip_piece_position(side, piece, to);

                    // Early return
                    return Ok((
                        board_new,
                        state_new,
                        zhash_new,
                        halfmove_clock_new,
                        fullmove_number_new,
                    ));
                }

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
                            // Castling king side
                            // Move rook
                            board_new.clear_piece(cstl_ks_r_from_square);
                            board_new.set_piece(side, Piece::Rook, cstl_ks_r_to_square);
                            zhash_new.flip_piece_position(side, Piece::Rook, cstl_ks_r_from_square);
                            zhash_new.flip_piece_position(side, Piece::Rook, cstl_ks_r_to_square);
                        }
                        to if to == cstl_qs_k_square => {
                            // Castling queen side
                            // Move rook
                            board_new.clear_piece(cstl_qs_r_from_square);
                            board_new.set_piece(side, Piece::Rook, cstl_qs_r_to_square);
                            zhash_new.flip_piece_position(side, Piece::Rook, cstl_qs_r_from_square);
                            zhash_new.flip_piece_position(side, Piece::Rook, cstl_qs_r_to_square);
                        }
                        _ => {}
                    }
                }

                // Move our piece
                board_new.clear_piece(from);
                board_new.set_piece(side, piece, to);
                zhash_new.flip_piece_position(side, piece, from);
                zhash_new.flip_piece_position(side, piece, to);
            }
            Piece::Knight | Piece::Bishop | Piece::Queen => {
                // Is capture?
                if (BitBoard::square(to) & opponent_pieces).not_empty() {
                    let (_, captured_piece) = board_new.piece_at(to).unwrap();

                    // Capture opponents piece
                    board_new.clear_piece(to);
                    zhash_new.flip_piece_position(opponent_side, captured_piece, to);

                    // Reset halfmove clock
                    halfmove_clock_new = 0;
                }

                // Move our piece
                board_new.clear_piece(from);
                board_new.set_piece(side, piece, to);
                zhash_new.flip_piece_position(side, piece, from);
                zhash_new.flip_piece_position(side, piece, to);
            }
            Piece::Rook => {
                // Is capture?
                if (BitBoard::square(to) & opponent_pieces).not_empty() {
                    let (_, captured_piece) = board_new.piece_at(to).unwrap();

                    // Capture opponents piece
                    board_new.clear_piece(to);
                    zhash_new.flip_piece_position(opponent_side, captured_piece, to);

                    // Reset halfmove clock
                    halfmove_clock_new = 0;
                }

                // Revoke castling rights
                let (king_side_origin, queen_side_origin) = match side {
                    Side::White => (Square::H1, Square::A1),
                    Side::Black => (Square::H8, Square::A8),
                };

                if from == king_side_origin && self.state.king_side_castling_right(side) {
                    state_new.set_king_side_castling_right(side, false);
                    zhash_new.flip_king_castling_right(side);
                } else if from == queen_side_origin && self.state.queen_side_castling_right(side) {
                    state_new.set_queen_side_castling_right(side, false);
                    zhash_new.flip_queen_castling_right(side);
                }

                // Move our piece
                board_new.clear_piece(from);
                board_new.set_piece(side, piece, to);
                zhash_new.flip_piece_position(side, piece, from);
                zhash_new.flip_piece_position(side, piece, to);
            }
        }

        Ok((
            board_new,
            state_new,
            zhash_new,
            halfmove_clock_new,
            fullmove_number_new,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_pseudo_legal_move_pawn_c2_to_d2() {
        let mov = Mov::new(10.into(), 18.into(), None);

        Game::new(Game::FEN_NEW_GAME)?.make_pseudo_legal_move(mov)?;
    }
}
