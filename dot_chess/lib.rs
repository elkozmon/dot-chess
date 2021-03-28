#![cfg_attr(not(feature = "std"), no_std)]

mod board;
mod zobrist;

use ink_lang as ink;

#[ink::contract]
mod dot_chess {

    use crate::board::{Board, Flags as BoardFlags, Piece, Ply, PlyFlags, Side, Square};
    use crate::zobrist::ZobristHash;
    use ink_storage::Vec;
    use scale::{Decode, Encode};

    #[derive(Encode, Decode, Debug, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidArgument,
        InvalidCaller,
        IllegalMove,
    }

    #[derive(Encode, Decode, Debug, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum GameOverReason {
        Checkmate,
        Stalemate,
        InsufficientMatingMaterial,
        Resignation,
        Repetition,
        FiftyMoveRule,
    }

    #[ink(event)]
    pub struct PlayerMoved {
        #[ink(topic)]
        side: Side,
        from: Square,
        to: Square,
    }

    #[ink(event)]
    pub struct GameOver {
        winner: Option<Side>,
        reason: GameOverReason,
    }

    #[ink(storage)]
    pub struct DotChess {
        /// Account playing as white
        white: AccountId,
        /// Account playing as black
        black: AccountId,
        /// Chess board
        board: ink_storage::Pack<Board>,
        /// Board history up to last capture or pawn movement
        board_history: Vec<ZobristHash>,
    }

    impl DotChess {
        /// Initiates new game
        #[ink(constructor)]
        pub fn new(white: AccountId, black: AccountId) -> Self {
            let board = Board::default();

            let mut board_history = Vec::new();
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
        pub fn get_board(&self) -> ([i8; 64], BoardFlags) {
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

                board[square.index() as usize] = n;
            }

            let flags = self.board.get_flags();

            (board, *flags)
        }

        /// Makes a move
        ///
        /// Returns true if move was successful
        #[ink(message)]
        pub fn make_move(&mut self, from: Square, to: Square, flags: PlyFlags) -> bool {
            if !self.is_callers_turn() {
                todo!("Emit event: Invalid caller");
                return false;
            }

            let side = self.board.get_side_turn();
            let ply = Ply::new(from, to, flags);

            match self.board.try_make_move(ply) {
                Ok((board_new, events)) => {
                    // Update board
                    self.board = ink_storage::Pack::new(board_new);

                    let opponent_side = side.flip();
                    let opponent_has_legal_moves = self.board.side_has_legal_move(opponent_side);
                    if !opponent_has_legal_moves {
                        let opponent_king_square = self.board.get_king_square(opponent_side);

                        if self.board.is_attacked(opponent_king_square, side) {
                            // Checkmate
                            // TODO Announce result
                        } else {
                            // Stalemate
                            // TODO Announce result
                        }
                    }

                    // Is insufficient mating material?
                    let mut white_score = 0;
                    let mut black_score = 0;
                    let mut insufficient_mating_material = true;

                    for (side, piece, square) in self.board.get_pieces().iter() {
                        let ref_score = match side {
                            Side::White => &mut white_score,
                            Side::Black => &mut black_score,
                        };

                        match piece {
                            Piece::Knight | Piece::Bishop => *ref_score += 1,
                            Piece::Pawn | Piece::Rook | Piece::Queen => *ref_score = i32::MAX,
                            Piece::King => {}
                        }

                        drop(ref_score);

                        if white_score > 1 && black_score > 1 {
                            insufficient_mating_material = false;
                            break;
                        }
                    }

                    if insufficient_mating_material {
                        // TODO Announce result
                    }

                    // Clear board history to save space
                    if self.board.halfmove_clock() == 0 {
                        self.board_history.clear()
                    }

                    // Is repetition?
                    let new_hash = self.board_history.last().unwrap().apply(events);

                    let is_repetition = self
                        .board_history
                        .iter()
                        .filter(|hash| hash == new_hash)
                        .take(2)
                        .count()
                        == 2;

                    if is_repetition {
                        // TODO Announce result
                    }

                    // Update history
                    self.board_history.push(new_hash);

                    // TODO Emit events

                    true
                }
                Err(error) => {
                    todo!("Emit event");

                    false
                }
            }
        }

        pub fn claim_draw(&mut self, reason: GameOverReason) -> bool {
            if !self.is_callers_turn() {
                todo!("Emit event: Invalid caller");
                return false;
            }

            match reason {
                GameOverReason::FiftyMoveRule if self.board.halfmove_clock() >= 100 => {
                    // TODO Announce draw
                }
                _ => {
                    panic!("Invalid claim")
                }
            }
        }

        pub fn resign(&mut self) -> bool {
            if !self.is_callers_turn() {
                todo!("Emit event: Invalid caller");
                return false;
            }

            // TODO Announce result
        }

        fn is_callers_turn(&self) -> bool {
            let caller_account = self.env().caller();

            // Assert it's callers turn
            let side = self.board.get_side_turn();
            let side_account = match side {
                Side::White => self.white,
                Side::Black => self.black,
            };

            caller_account == side_account
        }
    }
}
