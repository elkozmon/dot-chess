#![cfg_attr(not(feature = "std"), no_std)]
#![feature(iter_advance_by)]

extern crate alloc;

mod board;
mod common;
mod game;
mod gameover;
mod zobrist;

pub use crate::board::Mov;
pub use crate::common::{Error, Result};
pub use crate::game::Game;

use ink_lang as ink;

#[ink::contract]
mod dotchess {

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
        /// Blocks left for white
        white_blocks_left: u32,
        /// Blocks left for black
        black_blocks_left: u32,
        /// Block increment per move
        block_increment: u32,
        /// Block of last move
        last_move_block: BlockNumber,
    }

    impl DotChess {
        /// Initiates new game
        #[ink(constructor)]
        pub fn new(
            white: AccountId,
            black: AccountId,
            block_base: u32,
            block_increment: u32,
        ) -> Self {
            Self::from_fen(
                white,
                black,
                block_base,
                block_increment,
                Game::FEN_NEW_GAME.into(),
            )
        }

        /// Initiates game from given FEN
        #[ink(constructor)]
        pub fn from_fen(
            white: AccountId,
            black: AccountId,
            block_base: u32,
            block_increment: u32,
            fen: String,
        ) -> Self {
            let game = Game::new(fen.as_str()).unwrap();

            let mut zobrist = Vec::new();
            zobrist.push(game.zobrist());

            Self {
                white,
                black,
                game: Pack::new(game),
                zobrist: Pack::new(Box::new(zobrist)),
                white_blocks_left: block_base,
                black_blocks_left: block_base,
                block_increment,
                last_move_block: Self::env().block_number(),
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
            let next_side = self.game.side_next_in_turn();

            if !self.side_belongs_to_caller(next_side) {
                return Err(Error::InvalidCaller);
            }

            if !self.side_has_blocks_left(next_side) {
                return self.terminate_game_out_of_blocks(next_side);
            }

            let moov: Mov = mov.as_str().try_into()?;

            // Make move
            let game_new = self.game.make_move(&moov)?;

            // Opponent out of moves?
            if !game_new.has_legal_moves() {
                if game_new.is_check() {
                    return self.terminate_game(Some(next_side), GameOverReason::Checkmate);
                }

                return self.terminate_game(None, GameOverReason::Stalemate);
            }

            // Is insufficient mating material?
            if self.game.no_side_have_sufficient_mating_material() {
                return self.terminate_game(None, GameOverReason::InsufficientMatingMaterial);
            }

            // If halfmove clock resets, clear zobrist history
            if game_new.halfmove_clock() == 0 {
                self.zobrist.clear();
            }

            // Add zobrist to history
            self.zobrist.push(game_new.zobrist());

            // Emit event
            self.env().emit_event(PlayerMoved {
                side: next_side.into(),
                mov,
                fen: game_new.fen()?,
            });

            // Update blocks left (must go before updating block number)
            let block_diff = self.block_diff_since_last_move();

            let blocks_left_ref = match next_side {
                Side::White => &mut self.white_blocks_left,
                Side::Black => &mut self.black_blocks_left,
            };

            if game_new.fullmove_number() > 40 {
                *blocks_left_ref += self.block_increment;
            }

            *blocks_left_ref -= block_diff;

            // Update game and last move block number
            self.game = ink_storage::Pack::new(game_new);
            self.last_move_block = self.env().block_number();

            // Check if player has no blocks left after this move
            if !self.side_has_blocks_left(next_side) {
                return self.terminate_game_out_of_blocks(next_side);
            }

            Ok(())
        }

        #[ink(message)]
        pub fn report_abandonment(&mut self) -> Result<()> {
            let next_side = self.game.side_next_in_turn();

            if !self.side_has_blocks_left(next_side) {
                return self.terminate_game_out_of_blocks(next_side);
            }

            let error_message = format!(
                "{} not out of blocks",
                <Side as Into<&'static str>>::into(next_side)
            );

            Err(Error::InvalidArgument(error_message))
        }

        #[ink(message)]
        pub fn claim_draw(&mut self, reason: u8) -> Result<()> {
            let next_side = self.game.side_next_in_turn();

            if !self.side_belongs_to_caller(next_side) {
                return Err(Error::InvalidCaller);
            }

            if !self.side_has_blocks_left(next_side) {
                self.terminate_game_out_of_blocks(next_side)?;
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
                    "Draw claim on basis of {:?} doesn't meet the requirements",
                    reason
                ))),
            }
        }

        #[ink(message)]
        pub fn resign(&mut self) -> Result<()> {
            let next_side = self.game.side_next_in_turn();

            if !self.side_belongs_to_caller(next_side) {
                return Err(Error::InvalidCaller);
            }

            self.terminate_game(Some(next_side.flip()), GameOverReason::Resignation)
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

        fn terminate_game_out_of_blocks(&mut self, out_of_blocks_side: Side) -> Result<()> {
            let opponent_side = out_of_blocks_side.flip();

            if self.game.side_has_sufficient_mating_material(opponent_side) {
                return self.terminate_game(Some(opponent_side), GameOverReason::Abandonment);
            }

            return self.terminate_game(None, GameOverReason::Abandonment);
        }

        fn is_repetition(&self) -> bool {
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

        fn side_account(&self, side: Side) -> AccountId {
            match side {
                Side::White => self.white,
                Side::Black => self.black,
            }
        }

        fn side_has_next_turn(&self, side: Side) -> bool {
            side as u8 == self.game.side_next_in_turn() as u8
        }

        fn side_belongs_to_caller(&self, side: Side) -> bool {
            self.env().caller() == self.side_account(side)
        }

        fn side_has_blocks_left(&self, side: Side) -> bool {
            let mut blocks_left = match side {
                Side::White => self.white_blocks_left,
                Side::Black => self.black_blocks_left,
            };

            if self.side_has_next_turn(side) {
                return blocks_left >= self.block_diff_since_last_move();
            }

            blocks_left > 0
        }

        fn block_diff_since_last_move(&self) -> u32 {
            self.env().block_number() - self.last_move_block
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

            let mut chess = DotChess::new(white, black, 1, 1);

            chess.make_move("d2d3".to_string()).unwrap();
            chess.make_move("g7g6".to_string()).unwrap();
        }
    }
}
