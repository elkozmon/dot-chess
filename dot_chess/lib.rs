#![cfg_attr(not(feature = "std"), no_std)]

mod board;
mod event;
mod zobrist;

use ink_lang as ink;

#[ink::contract]
mod dot_chess {

    use crate::board::{Board, Piece, Player, Ply, PlyFlags, Square, SquareIndex};
    use crate::zobrist::ZobristHash;
    use ink_storage::collections::SmallVec;
    use scale::{Decode, Encode};

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidArgument,
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
        /// Is it whites turn?
        whites_turn: bool,
        /// Halfmove clock
        halfmove_clock: u8,
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
                whites_turn: true,
                halfmove_clock: 0,
                board_history,
            }
        }

        /// Returns array of 64 8-bit integers representing current state of the board
        /// in following square order: A1, A2, ..., B1, B2, ..., H8
        ///
        /// 0 - Empty square
        /// 1 - Pawn
        /// 2 - Knight
        /// 3 - Bishop
        /// 4 - Rook
        /// 5 - Queen
        /// 6 - King
        ///
        /// Negative integers represent black pieces
        /// Positive integers represent white pieces
        #[ink(message)]
        pub fn get_board(&self) -> [i8; 64] {
            let mut board = [0; 64];

            for (player, piece, square) in self.board.get_pieces().iter() {
                let n = match piece {
                    Piece::Pawn => 1,
                    Piece::Knight => 2,
                    Piece::Bishop => 3,
                    Piece::Rook => 4,
                    Piece::Queen => 5,
                    Piece::King => 6,
                };

                let n = match player {
                    Player::White => n,
                    Player::Black => -n,
                };

                board[square.to_index() as usize] = n;
            }

            board
        }

        /// Makes a move
        ///
        /// Returns true if move was successful
        #[ink(message)]
        pub fn make_move(&mut self, from: SquareIndex, to: SquareIndex, flags: PlyFlags) -> bool {
            let caller = self.env().caller();

            let account_in_turn = if self.whites_turn {
                self.white
            } else {
                self.black
            };

            // Only player in turn is allowed to call this
            if caller != account_in_turn {
                todo!("Emit event");

                return false;
            }

            let m = Ply::new(Square::from_index(from), Square::from_index(to), flags);

            match self.board.make_move(m) {
                Ok(events) => {
                    let new_hash = self.board_history.last().unwrap().apply(events);

                    self.board_history.push(new_hash);
                }
                Err(_) => {
                    todo!("Emit event");

                    return false;
                }
            }

            todo!("Check halfmove clock");
            if self.board_history.len() == self.board_history.capacity() {
                // Draw
                todo!("Emit event")
            }

            true
        }

        /// Returns true if it's whites turn, false otherwise
        #[ink(message)]
        pub fn get_whites_turn(&self) -> bool {
            self.whites_turn
        }
    }
}
