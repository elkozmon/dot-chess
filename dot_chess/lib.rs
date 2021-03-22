#![cfg_attr(not(feature = "std"), no_std)]

mod board;

use ink_lang as ink;

#[ink::contract]
mod dot_chess {

    use crate::board::{Board, Move, MoveFlags, Square, SquareIndex, MoveEncoded};
    use ink_storage::collections::SmallVec;
    use scale::{Decode, Encode};

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidArgument,
    }

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
        /// History of moves up to `halfmove_clock` long
        move_history: SmallVec<MoveEncoded, 50>
    }

    impl DotChess {
        /// Initiates new game
        #[ink(constructor)]
        pub fn new(white: AccountId, black: AccountId) -> Self {
            Self {
                white,
                black,
                board: ink_storage::Pack::new(Board::default()),
                whites_turn: true,
                halfmove_clock: 0,
                move_history: SmallVec::new()
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
        pub fn get_board(&self) -> [i8; 64] {
            self.board.to_array()
        }

        /// Makes a move
        ///
        /// Returns true if move was successful
        #[ink(message)]
        pub fn make_move(&self, from: SquareIndex, to: SquareIndex, flags: MoveFlags) -> bool {
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

            let m = Move::new(Square::from_index(from), Square::from_index(to), flags);

            todo!("Update board");

            true
        }

        /// Returns true if it's whites turn, false otherwise
        #[ink(message)]
        pub fn get_whites_turn(&self) -> bool {
            self.whites_turn
        }
    }
}
