mod bitboard;
mod direction;
mod file;
mod piece;
mod ply;
mod rank;
mod side;
mod square;

use crate::{dot_chess::Error, zobrist::ZobristHash};
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use ink_storage::Vec;
use scale::{Decode, Encode};

pub use bitboard::BitBoard;
pub use direction::Direction;
pub use file::File;
pub use piece::Piece;
pub use ply::Ply;
pub use rank::Rank;
pub use side::Side;
pub use square::Square;

#[derive(Copy, Clone, Encode, Decode, SpreadLayout, PackedLayout)]
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

impl core::convert::Into<ZobristHash> for Board {
    fn into(self) -> ZobristHash {
        let mut zhash = ZobristHash::zero();

        for (side, piece, square) in self.pieces().iter() {
            zhash.flip_piece_position(*side, *piece, *square);
        }

        zhash
    }
}

impl Board {
    pub fn empty() -> Self {
        Self {
            black: BitBoard::EMPTY,
            white: BitBoard::EMPTY,
            kings: BitBoard::EMPTY,
            queens: BitBoard::EMPTY,
            rooks: BitBoard::EMPTY,
            bishops: BitBoard::EMPTY,
            knights: BitBoard::EMPTY,
            pawns: BitBoard::EMPTY,
        }
    }

    pub fn pieces(&self) -> Vec<(Side, Piece, Square)> {
        let mut pieces = Vec::new();
        let mut occupied = self.occupied();

        while occupied.not_empty() {
            let square = occupied.pop_square();

            if let Some((side, piece)) = self.piece_at(square) {
                pieces.push((side, piece, square));
            }
        }

        pieces
    }

    pub fn is_king_attacked(&self, king_side: Side) -> bool {
        self.is_attacked(self.king_square(king_side), king_side.flip())
    }

    fn king_square(&self, side: Side) -> Square {
        let pieces = match side {
            Side::White => self.white,
            Side::Black => self.black,
        };

        (pieces & self.kings).pop_square()
    }

    // TODO test
    fn is_attacked(&self, square: Square, by_side: Side) -> bool {
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

    pub fn occupied(&self) -> BitBoard {
        self.black | self.white
    }

    pub fn ray_attacks(&self, from: Square, direction: Direction) -> BitBoard {
        let mut attacks = BitBoard::ray_mask(from, direction);
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

    pub fn diagonal_attacks(&self, from: Square) -> BitBoard {
        self.ray_attacks(from, Direction::NorthEast) | self.ray_attacks(from, Direction::SouthWest)
    }

    pub fn anti_diagonal_attacks(&self, from: Square) -> BitBoard {
        self.ray_attacks(from, Direction::NorthWest) | self.ray_attacks(from, Direction::SouthEast)
    }

    pub fn file_attacks(&self, from: Square) -> BitBoard {
        self.ray_attacks(from, Direction::North) | self.ray_attacks(from, Direction::South)
    }

    pub fn rank_attacks(&self, from: Square) -> BitBoard {
        self.ray_attacks(from, Direction::East) | self.ray_attacks(from, Direction::West)
    }

    pub fn rook_attacks(&self, from: Square) -> BitBoard {
        self.file_attacks(from) | self.rank_attacks(from)
    }

    pub fn bishop_attacks(&self, from: Square) -> BitBoard {
        self.diagonal_attacks(from) | self.anti_diagonal_attacks(from)
    }

    pub fn queen_attacks(&self, from: Square) -> BitBoard {
        self.rook_attacks(from) | self.bishop_attacks(from)
    }

    // TODO test
    pub fn pseudo_legal_moves_from(
        &self,
        from: Square,
        king_castling_right: bool,
        queen_castling_right: bool,
        en_passant_files: Vec<File>,
    ) -> BitBoard {
        match self.piece_at(from) {
            None => BitBoard::EMPTY,
            Some((side, piece)) => {
                let not_own_pieces = !self.pieces_by_side(side);

                match (side, piece) {
                    (_, Piece::Bishop) => self.bishop_attacks(from) & not_own_pieces,
                    (_, Piece::Rook) => self.rook_attacks(from) & not_own_pieces,
                    (_, Piece::Queen) => self.queen_attacks(from) & not_own_pieces,
                    (_, Piece::Knight) => BitBoard::knight_attacks_mask(from) & not_own_pieces,
                    (_, Piece::King) => {
                        let not_occuppied = !self.occupied();
                        let king = BitBoard::square(from);

                        let mut castling_queen_side = BitBoard::EMPTY;

                        if queen_castling_right {
                            castling_queen_side |= (king.west_one() & not_occuppied).west_one()
                                & not_occuppied
                                & BitBoard::FILE_C;
                        }

                        let mut castling_king_side = BitBoard::EMPTY;

                        if king_castling_right {
                            castling_king_side |= (king.east_one() & not_occuppied).east_one()
                                & not_occuppied
                                & BitBoard::FILE_G;
                        }

                        (BitBoard::king_attacks_mask(from) & not_own_pieces)
                            | castling_king_side
                            | castling_queen_side
                    }
                    (Side::White, Piece::Pawn) => {
                        let pawn: BitBoard = self.white & BitBoard::square(from);

                        let black_pawns: BitBoard = self.black & self.pawns;
                        let any_attacks: BitBoard = pawn.white_pawn_any_attacks_mask();
                        let sgl_targets: BitBoard = pawn.north_one() & not_own_pieces;
                        let any_targets: BitBoard =
                            sgl_targets | sgl_targets.north_one() & BitBoard::RANK_4;

                        let pas_targets: BitBoard = BitBoard::RANK_6
                            & en_passant_files
                                .iter()
                                .fold(BitBoard::EMPTY, |bb, file| bb & BitBoard::from(*file));

                        ((any_attacks & (pas_targets | black_pawns)) | any_targets) & not_own_pieces
                    }
                    (Side::Black, Piece::Pawn) => {
                        let pawn: BitBoard = self.black & BitBoard::square(from);

                        let white_pawns: BitBoard = self.white & self.pawns;
                        let any_attacks: BitBoard = pawn.black_pawn_any_attacks_mask();
                        let sgl_targets: BitBoard = pawn.south_one() & not_own_pieces;
                        let any_targets: BitBoard =
                            sgl_targets | sgl_targets.south_one() & BitBoard::RANK_5;

                        let pas_targets: BitBoard = BitBoard::RANK_3
                            & en_passant_files
                                .iter()
                                .fold(BitBoard::EMPTY, |bb, file| bb & BitBoard::from(*file));

                        ((any_attacks & (pas_targets | white_pawns)) | any_targets) & not_own_pieces
                    }
                }
            }
        }
    }

    pub fn set_piece(&mut self, side: Side, piece: Piece, square: Square) {
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

    pub fn clear_piece(&mut self, square: Square) {
        let not_bb = !BitBoard::square(square);

        self.black &= not_bb;
        self.white &= not_bb;
        self.knights &= not_bb;
        self.bishops &= not_bb;
        self.rooks &= not_bb;
        self.queens &= not_bb;
        self.kings &= not_bb;
    }

    pub fn pieces_by_side(&self, side: Side) -> BitBoard {
        match side {
            Side::White => self.white,
            Side::Black => self.black,
        }
    }

    pub fn piece_at(&self, square: Square) -> Option<(Side, Piece)> {
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
    fn ray_attacks_sw_g5() {
        let mut board = Board::empty();
        let square = Square::new(File::G, Rank::_5);

        let pieces = [(Side::White, Piece::Queen, square)];
        for (side, piece, square) in pieces.iter() {
            board.set_piece(*side, *piece, *square);
        }

        assert_eq!(
            board.ray_attacks(square, Direction::SouthWest),
            0x20100800.into()
        );
    }

    #[test]
    fn ray_attacks_n_d5() {
        let mut board = Board::empty();
        let square = Square::new(File::D, Rank::_5);

        let pieces = [(Side::White, Piece::Queen, square)];
        for (side, piece, square) in pieces.iter() {
            board.set_piece(*side, *piece, *square);
        }

        assert_eq!(
            board.ray_attacks(square, Direction::North),
            0x8080000000000.into()
        );
    }

    #[test]
    fn ray_attacks_sw_h8() {
        let mut board = Board::empty();
        let square = Square::new(File::H, Rank::_8);

        let pieces = [(Side::White, Piece::Rook, Square::new(File::A, Rank::_1))];
        for (side, piece, square) in pieces.iter() {
            board.set_piece(*side, *piece, *square);
        }

        assert_eq!(
            board.ray_attacks(square, Direction::SouthWest),
            0x40201008040201.into()
        );
    }

    #[test]
    fn board_get_pieces() {
        let mut board = Board::empty();

        let pieces = vec![
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
        ];

        for (side, piece, square) in pieces.iter() {
            board.set_piece(*side, *piece, *square);
        }

        assert_eq!(board.pieces(), pieces.into_iter().collect());
    }
}
