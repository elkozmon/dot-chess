#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod board;
mod game;
mod gameover;
mod zobrist;

use ink_lang as ink;

#[ink::contract]
mod dot_chess {

    use crate::board::{Board, Piece, Ply, Side, Square};
    use crate::game::Game;
    use crate::gameover::GameOverReason;
    use crate::zobrist::ZobristHash;
    use alloc::string::String;
    use core::convert::TryInto;
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

    #[ink(event)]
    pub struct PlayerMoved {
        #[ink(topic)]
        side: u8,
        mov: String,
        fen: String,
    }

    #[ink(event)]
    pub struct GameOver {
        winner: Option<u8>,
        reason: u8,
    }

    #[ink(storage)]
    pub struct DotChess {
        /// Account playing as white
        white: AccountId,
        /// Account playing as black
        black: AccountId,
        /// Match state
        game: ink_storage::Pack<Game>,
    }

    impl DotChess {
        /// Initiates new game
        #[ink(constructor)]
        pub fn new(white: AccountId, black: AccountId) -> Self {
            let game = ink_storage::Pack::new(Game::new());

            Self { white, black, game }
        }

        /// Returns FEN string representation of current game
        #[ink(message)]
        pub fn fen(&self) -> &'static str {
            todo!()
        }

        /// Makes a move
        ///
        /// Returns true if move was successful
        #[ink(message)]
        pub fn make_move(&mut self, from: u8, to: u8, promotion: Option<u8>) -> Result<()> {
            if !self.is_callers_turn() {
                return Err(Error::InvalidCaller);
            }

            // TODO rewrite to string arg
            let from: Square = from.into();
            let to: Square = to.into();
            let promotion: Option<Piece> = match promotion {
                Some(val) => Some(val.try_into()?),
                None => None,
            };

            let side = self.game.next_turn_side();
            let ply = Ply::new(from, to, promotion);

            // Make move
            let game_new = self.game.make_move(ply)?;

            // Opponent out of moves?
            if !game_new.has_legal_moves() {
                if game_new.is_check() {
                    return self.terminate_game(Some(side), GameOverReason::Checkmate);
                }

                return self.terminate_game(None, GameOverReason::Stalemate);
            }

            // Is insufficient mating material?
            if !game_new.has_sufficient_mating_material() {
                return self.terminate_game(None, GameOverReason::InsufficientMatingMaterial);
            }

            // Update game
            self.game = ink_storage::Pack::new(game_new);

            // Emit event
            // TODO
            self.env().emit_event(PlayerMoved {
                side: side.into(),
                mov: String::from(""),
                fen: String::from(""),
            });

            Ok(())
        }

        #[ink(message)]
        pub fn claim_draw(&mut self, reason: u8) -> Result<()> {
            if !self.is_callers_turn() {
                return Err(Error::InvalidCaller);
            }

            let reason: GameOverReason = reason.try_into()?;

            match reason {
                GameOverReason::FiftyMoveRule if self.game.halfmove_clock() >= 100 => {
                    self.terminate_game(None, GameOverReason::FiftyMoveRule)
                }
                GameOverReason::Repetition if self.game.is_repetition() => {
                    self.terminate_game(None, GameOverReason::Repetition)
                }
                _ => Err(Error::InvalidArgument),
            }
        }

        #[ink(message)]
        pub fn resign(&mut self) -> Result<()> {
            if !self.is_callers_turn() {
                return Err(Error::InvalidCaller);
            }

            let resignee_side = self.game.next_turn_side();

            self.terminate_game(Some(resignee_side.flip()), GameOverReason::Resignation)
        }

        fn terminate_game(&mut self, winner: Option<Side>, reason: GameOverReason) -> Result<()> {
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

            let winner: Option<u8> = winner.map(|side| side.into());
            let reason: u8 = reason.into();

            self.env().emit_event(GameOver { winner, reason });
            self.env().terminate_contract(FEE_BENEFICIARY.into())
        }

        fn is_callers_turn(&self) -> bool {
            let caller_account = self.env().caller();

            // Assert it's callers turn
            let side = self.game.next_turn_side();
            let side_account = match side {
                Side::White => self.white,
                Side::Black => self.black,
            };

            caller_account == side_account
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_env::AccountId;
        use ink_lang as ink;

        #[ink::test]
        fn make_move() {
            let white = AccountId::from([0x01; 32]);
            let black = AccountId::from([0x01; 32]);

            let mut chess = DotChess::new(white, black);

            chess.make_move(10, 18, None).unwrap();
            chess.make_move(53, 45, None).unwrap();
        }
    }
}
