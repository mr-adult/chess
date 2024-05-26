use chess_common::{File, Location, Rank};
use env_logger::Env;

mod bitboard;
mod board;
mod chess_move;
mod piece;

use board::Board;
use chess_move::Move;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    println!("{}", log::Level::max());
    let board = Board::default();
    println!("{}", board);
}
