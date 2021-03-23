#![cfg_attr(not(feature = "std"), no_std)]

mod board;
mod event;
mod zobrist;

use ink_lang as ink;

#[ink::contract]
mod dot_chess {

    use crate::{
        board::{Board, Flags as BoardFlags, Piece, Ply, PlyFlags, Side, Square, SquareIndex},
        event::Event,
        zobrist::ZobristHash,
    };
    use ink_storage::collections::SmallVec;
    use scale::{Decode, Encode};

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidArgument,
        InvalidCaller,
    }

    const BOARD_HISTORY_SIZE: usize = 100;

    #[ink(storage)]
    pub struct DotChess {
        /// Account playing as white
        white: AccountId,
        /// Account playing as black
        black: AccountId,
        /// Chess board
        board: ink_storage::Pack<Board>,
        /// Board history of up to 100 states
        board_history: SmallVec<ZobristHash, BOARD_HISTORY_SIZE>,
    }

    impl DotChess {
        /// Initiates new game
        #[ink(constructor)]
        pub fn new(white: AccountId, black: AccountId) -> Self {
            let board = Board::default();

            let mut board_history = SmallVec::new();
            let zobrist_hash = ZobristHash::new(&board);
            board_history.push(zobrist_hash);

            Self {
                white,
                black,
                board: ink_storage::Pack::new(board),
                board_history,
            }
        }

        /// Returns array of 64 8-bit integers describing positions on the board, and a unsigned 16-bit integer with game state flags
        ///
        /// Positions are described in order of squares from A1, A2, ..., B1, B2, ... H8 and encoded using these codes:
        ///
        ///   0 - Empty square
        ///   1 - Pawn
        ///   2 - Knight
        ///   3 - Bishop
        ///   4 - Rook
        ///   5 - Queen
        ///   6 - King
        ///
        /// Positive integers represent white pieces
        /// Negative integers represent black pieces
        ///
        /// The unsined 16-bit integer is a game state bit mask:
        ///
        ///   1 << 0  En passant open at file A
        ///   1 << 1  En passant open at file B
        ///   1 << 2  En passant open at file C
        ///   1 << 3  En passant open at file D
        ///   1 << 4  En passant open at file E
        ///   1 << 5  En passant open at file F
        ///   1 << 6  En passant open at file G
        ///   1 << 7  En passant open at file H
        ///   1 << 8  White Queen Castling Right
        ///   1 << 9  White King Castling Right
        ///   1 << 10 Black Queen Castling Right
        ///   1 << 11 Black King Castling Right
        ///   1 << 12 Whites Turn
        #[ink(message)]
        pub fn get_board(&self) -> ([i8; 64], &BoardFlags) {
            let mut board = [0i8; 64];

            for (side, piece, square) in self.board.get_pieces().iter() {
                let n = match piece {
                    Piece::Pawn => 1,
                    Piece::Knight => 2,
                    Piece::Bishop => 3,
                    Piece::Rook => 4,
                    Piece::Queen => 5,
                    Piece::King => 6,
                };

                let n = match side {
                    Side::White => n,
                    Side::Black => -n,
                };

                board[square.to_index() as usize] = n;
            }

            let flags = self.board.get_flags();

            (board, flags)
        }

        fn _make_move(
            &mut self,
            from: SquareIndex,
            to: SquareIndex,
            flags: PlyFlags,
        ) -> Result<Vec<Event>, Error> {
            let caller = self.env().caller();

            // Assert it's callers turn
            let account_in_turn = if self.board.get_whites_turn() {
                self.white
            } else {
                self.black
            };

            if caller != account_in_turn {
                return Err(Error::InvalidCaller);
            }

            let ply = Ply::new(Square::from_index(from), Square::from_index(to), flags);

            Ok(self.board.make_move(ply)?)
        }

        /// Makes a move
        ///
        /// Returns true if move was successful
        #[ink(message)]
        pub fn make_move(&mut self, from: SquareIndex, to: SquareIndex, flags: PlyFlags) -> bool {
            match self._make_move(from, to, flags) {
                Ok(events) => {
                    let new_hash = self.board_history.last().unwrap().apply(events);

                    self.board_history.push(new_hash);

                    if self.board_history.len() == self.board_history.capacity() {
                        // Draw
                    }

                    true
                }
                Err(error) => {
                    todo!("Emit event");

                    false
                }
            }
        }
    }
}
