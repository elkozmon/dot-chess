#![cfg_attr(not(feature = "std"), no_std)]
#![feature(iter_advance_by)]

extern crate alloc;
extern crate num;

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
    use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
    use ink_storage::{Box, Pack, Vec};
    use scale::{Decode, Encode};

    const BALANCE_DISTRIBUTION_RATIO: Balance = 98;
    const FEE_BENEFICIARY: [u8; 32] = [
        212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88,
        133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
    ];

    #[ink(event)]
    pub struct DrawOfferUpdate {
        #[ink(topic)]
        side: String,
        offer: bool,
        fen: String,
    }

    #[ink(event)]
    pub struct BoardUpdate {
        #[ink(topic)]
        next_side: String,
        next_move_block_deadline: u32,
        last_move: String,
        last_side_blocks_left: u32,
        fen: String,
    }

    #[ink(event)]
    pub struct GameOver {
        winner: Option<String>,
        reason: String,
    }

    #[derive(Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, PartialEq, Eq, scale_info::TypeInfo, StorageLayout)
    )]
    struct Info {
        white_account: AccountId,
        black_account: AccountId,
        white_blocks_left: u32,
        black_blocks_left: u32,
        white_draw_offer: bool,
        black_draw_offer: bool,
        last_move_block: BlockNumber,
    }

    #[ink(storage)]
    pub struct DotChess {
        /// Game state
        game: Pack<Game>,
        /// Extra info
        info: Pack<Info>,
        /// Zobrist hash history
        zobrist: Pack<Box<Vec<ZobristHash>>>,
        /// Block increment per move
        block_increment: u32,
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

            let info = Info {
                white_account: white,
                black_account: black,
                white_blocks_left: block_base,
                black_blocks_left: block_base,
                white_draw_offer: false,
                black_draw_offer: false,
                last_move_block: Self::env().block_number(),
            };

            Self {
                game: Pack::new(game),
                info: Pack::new(info),
                zobrist: Pack::new(Box::new(zobrist)),
                block_increment,
            }
        }

        /// Returns FEN string representation of current game
        #[ink(message)]
        pub fn fen(&self) -> Result<String> {
            Ok(self.game.fen()?)
        }

        /// Returns number of blocks given side has left
        #[ink(message)]
        pub fn blocks_left(&self, side: String) -> Result<u32> {
            let side = Side::from_str(side)?;

            Ok(self.side_blocks_left(side))
        }

        /// Makes a move
        #[ink(message)]
        pub fn make_move(&mut self, mov: String) -> Result<()> {
            let us_side = self.game.side_next_in_turn();

            if !self.side_belongs_to_caller(us_side) {
                return Err(Error::InvalidCaller);
            }

            if self.side_blocks_left(us_side) == 0 {
                return self.terminate_game_out_of_blocks(us_side);
            }

            let moov: Mov = mov.as_str().try_into()?;

            // Make move
            let game_new = self.game.make_move(&moov)?;

            // Opponent out of moves?
            if !game_new.has_legal_moves() {
                if game_new.is_check() {
                    return self.terminate_game(Some(us_side), GameOverReason::Checkmate);
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

            // Update blocks left (must go before updating block number)
            let block_diff = self.block_diff_since_last_move();

            let blocks_left_ref = match us_side {
                Side::White => &mut self.info.white_blocks_left,
                Side::Black => &mut self.info.black_blocks_left,
            };

            if game_new.fullmove_number() > 40 {
                *blocks_left_ref += self.block_increment;
            }

            *blocks_left_ref -= block_diff;

            let last_side_blocks_left = *blocks_left_ref;

            // Update game and last move block number
            self.game = ink_storage::Pack::new(game_new);
            self.info.last_move_block = self.env().block_number();

            // Check if player has no blocks left after this move
            if self.side_blocks_left(us_side) == 0 {
                return self.terminate_game_out_of_blocks(us_side);
            }

            // Check for threefold repetition
            if self.max_repetition() >= 3 {
                return self.terminate_game(None, GameOverReason::ThreefoldRepetition);
            }

            // Check fifty move rule
            if self.game.halfmove_clock() >= 100 {
                return self.terminate_game(None, GameOverReason::FiftyMoveRule);
            }

            let op_side = us_side.flip();

            let next_move_block_deadline =
                self.side_blocks_left(op_side) + self.env().block_number();

            // Emit event
            self.env().emit_event(BoardUpdate {
                next_side: String::from(op_side.as_str()),
                next_move_block_deadline,
                last_move: mov,
                last_side_blocks_left,
                fen: self.game.fen()?,
            });

            Ok(())
        }

        /// Reports that side which turn it is has abandoned the match
        #[ink(message)]
        pub fn report_abandonment(&mut self) -> Result<()> {
            let next_side = self.game.side_next_in_turn();

            if self.side_blocks_left(next_side) == 0 {
                return self.terminate_game_out_of_blocks(next_side);
            }

            let error_message = format!("{} not out of blocks", String::from(next_side.as_str()));

            Err(Error::InvalidArgument(error_message))
        }

        #[ink(message)]
        pub fn offer_draw(&mut self, offer: bool) -> Result<()> {
            let next_side = self.game.side_next_in_turn();

            if !self.side_belongs_to_caller(next_side) {
                return Err(Error::InvalidCaller);
            }

            if self.side_blocks_left(next_side) == 0 {
                return self.terminate_game_out_of_blocks(next_side);
            }

            if offer && self.side_draw_offer(next_side.flip()) {
                return self.terminate_game(None, GameOverReason::DrawAgreement);
            }

            self.set_side_draw_offer(next_side, offer);

            self.env().emit_event(DrawOfferUpdate {
                side: String::from(next_side.as_str()),
                offer,
                fen: self.game.fen()?,
            });

            Ok(())
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
                Some(Side::White) => self.env().transfer(self.info.white_account, pot)?,
                Some(Side::Black) => self.env().transfer(self.info.black_account, pot)?,
                None => {
                    let split = pot / 2;
                    self.env().transfer(self.info.white_account, split)?;
                    self.env().transfer(self.info.black_account, split)?;
                }
            }

            let winner = winner.map(|side| String::from(side.as_str()));
            let reason = String::from(reason.as_str());

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

        fn max_repetition(&self) -> u32 {
            *self
                .zobrist
                .iter()
                .fold(HashMap::<ZobristHash, u32>::new(), |mut map, zhash| {
                    *map.entry(*zhash).or_insert(0) += 1;
                    map
                })
                .into_iter()
                .max_by_key(|(_, v)| *v)
                .map(|(_, v)| v)
                .unwrap()
        }

        fn side_draw_offer(&self, side: Side) -> bool {
            match side {
                Side::White => self.info.white_draw_offer,
                Side::Black => self.info.black_draw_offer,
            }
        }

        fn set_side_draw_offer(&mut self, side: Side, offer: bool) -> () {
            match side {
                Side::White => self.info.white_draw_offer = offer,
                Side::Black => self.info.black_draw_offer = offer,
            }
        }

        fn side_account(&self, side: Side) -> AccountId {
            match side {
                Side::White => self.info.white_account,
                Side::Black => self.info.black_account,
            }
        }

        fn side_has_next_turn(&self, side: Side) -> bool {
            side as u8 == self.game.side_next_in_turn() as u8
        }

        fn side_belongs_to_caller(&self, side: Side) -> bool {
            self.env().caller() == self.side_account(side)
        }

        fn side_blocks_left(&self, side: Side) -> u32 {
            let blocks_left = match side {
                Side::White => self.info.white_blocks_left,
                Side::Black => self.info.black_blocks_left,
            };

            if self.side_has_next_turn(side) {
                let block_diff = self.block_diff_since_last_move();
                let blocks_left_plus_1 = blocks_left + 1;

                if blocks_left_plus_1 < block_diff {
                    return 0;
                }

                return blocks_left_plus_1 - block_diff;
            }

            blocks_left
        }

        fn block_diff_since_last_move(&self) -> u32 {
            self.env().block_number() - self.info.last_move_block
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
