#![feature(iter_advance_by)]

extern crate alloc;

mod board;
mod common;
mod game;
mod zobrist;

use board::Mov;
use common::{Error, Result};
use game::Game;
use ink_lang as ink;
use std::{convert::TryInto, env};

pub fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let depth = args
        .get(1)
        .expect("depth argument missing")
        .parse::<usize>()
        .or_else(|_| {
            Err(Error::InvalidArgument(String::from(
                "depth must be a number",
            )))
        })?;

    assert!(depth > 0, "depth must be higher than 0");

    let fen = args.get(2).expect("fen argument missing");

    let mut game = Game::new(fen)?;

    if let Some(moves) = args.get(3) {
        for mov in moves.split_whitespace() {
            let mov: Mov = mov.try_into()?;
            game = game.make_move(&mov)?;
        }
    };

    let mut sum = 0;

    for mov in game.legal_moves().iter() {
        let game_new = game.make_move(mov)?;

        let n = if depth > 1 {
            perft(&game_new, depth - 1)?
        } else {
            1
        };

        println!("{} {}", mov, n);

        sum += n;
    }

    println!("\n{}", sum);

    Ok(())
}

fn perft(game: &Game, depth: usize) -> Result<u32> {
    let movs = game.legal_moves();

    if depth == 1 {
        return Ok(movs.len());
    }

    let mut nodes = 0;

    for mov in movs.iter() {
        let game_new = game.make_move(&mov)?;
        nodes += perft(&game_new, depth - 1)?;
    }

    Ok(nodes)
}
