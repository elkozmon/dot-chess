#![cfg_attr(not(feature = "std"), no_std)]
#![feature(iter_advance_by)]

extern crate alloc;

mod board;
mod common;
mod game;
mod gameover;
mod zobrist;

use ink_lang as ink;

#[ink::contract]
mod dot_chess {

    use crate::board::{Mov, Side};
    use crate::common::{Error, Result};
    use crate::game::Game;
    use crate::gameover::GameOverReason;
    use crate::zobrist::ZobristHash;
    use alloc::format;
    use alloc::string::String;
    use core::convert::TryInto;
    use ink_storage::collections::HashMap;
    use ink_storage::{Box, Pack, Vec};
    use scale::{Decode, Encode};

    const BALANCE_DISTRIBUTION_RATIO: Balance = 98;
    const FEE_BENEFICIARY: [u8; 32] = [
        212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88,
        133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
    ];

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
        /// Game state
        game: Pack<Game>,
        /// Zobrist hash history
        zobrist: Pack<Box<Vec<ZobristHash>>>,
    }

    impl DotChess {
        /// Initiates new game
        #[ink(constructor)]
        pub fn new(white: AccountId, black: AccountId) -> Self {
            Self::from_fen(white, black, Game::FEN_NEW_GAME.into())
        }

        /// Initiates game from given FEN
        #[ink(constructor)]
        pub fn from_fen(white: AccountId, black: AccountId, fen: String) -> Self {
            let game = Game::new(fen.as_str()).unwrap();

            let mut zobrist = Vec::new();
            zobrist.push(game.zobrist());

            Self {
                white,
                black,
                game: Pack::new(game),
                zobrist: Pack::new(Box::new(zobrist)),
            }
        }

        /// Returns FEN string representation of current game
        #[ink(message)]
        pub fn fen(&self) -> String {
            self.game.fen().unwrap()
        }

        /// Makes a move
        #[ink(message)]
        pub fn make_move(&mut self, mov: String) -> Result<()> {
            if !self.is_callers_turn() {
                return Err(Error::InvalidCaller);
            }

            let side = self.game.next_turn_side();
            let moov: Mov = mov.as_str().try_into()?;

            // Make move
            let game_new = self.game.make_move(&moov)?;

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

            // If halfmove clock resets, clear zobrist history
            if game_new.halfmove_clock() == 0 {
                self.zobrist.clear();
            }

            // Add zobrist to history
            self.zobrist.push(game_new.zobrist());

            // Update game
            self.game = ink_storage::Pack::new(game_new);

            // Emit event
            self.env().emit_event(PlayerMoved {
                side: side.into(),
                mov,
                fen: self.game.fen()?,
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
                GameOverReason::Repetition if self.is_repetition() => {
                    self.terminate_game(None, GameOverReason::Repetition)
                }
                reason => Err(Error::InvalidArgument(format!(
                    "Draw claim due to {:?} doesn't meet requirements",
                    reason
                ))),
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

        pub fn is_repetition(&self) -> bool {
            let occ = *self
                .zobrist
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

            chess.make_move("d2d3".to_string()).unwrap();
            chess.make_move("g7g6".to_string()).unwrap();
        }
    }
}
