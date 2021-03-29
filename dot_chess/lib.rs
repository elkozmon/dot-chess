#![cfg_attr(not(feature = "std"), no_std)]

mod board;
mod zobrist;

use ink_lang as ink;

#[ink::contract]
mod dot_chess {

    use crate::board::{Board, Piece, Ply, Side, Square};
    use crate::zobrist::ZobristHash;
    use ink_storage::Vec;
    use scale::{Decode, Encode};

    const BALANCE_DISTRIBUTION_RATIO: Balance = 98;
    const FEE_BENEFICIARY: [u8; 32] = [
        212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88,
        133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
    ];

    pub type Result<T> = core::result::Result<T, Error>;

    #[derive(Encode, Decode, Debug, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidArgument,
        InvalidCaller,
        IllegalMove,
        Other,
    }

    impl core::convert::From<ink_env::Error> for Error {
        fn from(_: ink_env::Error) -> Self {
            Self::Other
        }
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
        pub fn get_board(&self) -> ([i8; 64], u16) {
            let mut board = [0i8; 64];

            for (side, piece, square) in self.board.get_pieces().iter() {
                let n = <Piece as Into<u8>>::into(*piece) as i8;
                let n = match side {
                    Side::White => n,
                    Side::Black => -n,
                };

                board[square.index() as usize] = n;
            }

            let flags: u16 = (*self.board.get_flags()).into();

            (board, flags)
        }

        /// Makes a move
        ///
        /// Returns true if move was successful
        #[ink(message)]
        pub fn make_move(
            &mut self,
            from: Square,
            to: Square,
            promotion: Option<Piece>,
        ) -> Result<()> {
            if !self.is_callers_turn() {
                return Err(Error::InvalidCaller);
            }

            let side = self.board.get_side_turn();
            let ply = Ply::new(from, to, promotion);

            let (board_new, events) = self.board.try_make_move(ply)?;

            // Update board
            self.board = ink_storage::Pack::new(board_new);

            let opponent_side = side.flip();
            let opponent_has_legal_moves = self.board.side_has_legal_move(opponent_side);
            if !opponent_has_legal_moves {
                let opponent_king_square = self.board.get_king_square(opponent_side);

                if self.board.is_attacked(opponent_king_square, side) {
                    // Checkmate
                    return self.terminate_game(Some(side), GameOverReason::Checkmate);
                } else {
                    // Stalemate
                    return self.terminate_game(None, GameOverReason::Stalemate);
                }
            }

            // Is insufficient mating material?
            let mut white_score = 0;
            let mut black_score = 0;
            let mut insufficient_mating_material = true;

            for (side, piece, _) in self.board.get_pieces().iter() {
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
                return self.terminate_game(None, GameOverReason::InsufficientMatingMaterial);
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
                .filter(|hash| **hash == new_hash)
                .take(2)
                .count()
                == 2;

            if is_repetition {
                return self.terminate_game(None, GameOverReason::Repetition);
            }

            // Update history
            self.board_history.push(new_hash);

            // Emit event
            self.env().emit_event(PlayerMoved { side, from, to });

            Ok(())
        }

        #[ink(message)]
        pub fn claim_draw(&mut self, reason: GameOverReason) -> Result<()> {
            if !self.is_callers_turn() {
                return Err(Error::InvalidCaller);
            }

            match reason {
                GameOverReason::FiftyMoveRule if self.board.halfmove_clock() >= 100 => {
                    self.terminate_game(None, GameOverReason::FiftyMoveRule)
                }
                _ => Err(Error::InvalidArgument),
            }
        }

        #[ink(message)]
        pub fn resign(&mut self) -> Result<()> {
            if !self.is_callers_turn() {
                return Err(Error::InvalidCaller);
            }

            let resignee_side = self.board.get_side_turn();

            self.terminate_game(Some(resignee_side.flip()), GameOverReason::Resignation)
        }

        fn terminate_game(&mut self, winner: Option<Side>, reason: GameOverReason) -> Result<()> {
            self.env().emit_event(GameOver { winner, reason });

            let balance = self.env().balance();
            let fee = balance / BALANCE_DISTRIBUTION_RATIO;
            let pot = balance - fee;

            match winner {
                Some(Side::White) => self.env().transfer(self.white, pot)?,
                Some(Side::Black) => self.env().transfer(self.black, pot)?,
                None => {
                    let split = pot / 2;
                    self.env().transfer(self.white, split)?;
                    self.env().transfer(self.black, split)?;
                }
            }

            self.env().terminate_contract(FEE_BENEFICIARY.into())
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
