mod pgn_parser;
pub use pgn_parser::{ParsedGame, PgnErr};
use pgn_parser::PgnParser;

pub use fen_parser::{BoardLayout, FenErr};
use fen_parser::FenParser;

mod acn_parser;
mod fen_parser;

pub fn parse_pgn(pgn: &[u8]) -> Result<Vec<ParsedGame>, PgnErr> {
    PgnParser::parse_pgn(pgn)
}

pub fn parse_fen(fen: &str) -> Result<BoardLayout, FenErr> {
    FenParser::parse_fen(fen)
}
