pub use fen_parser::{BoardLayout, FenErr};
use fen_parser::FenParser;

mod fen_parser;

pub fn parse_fen(fen: &str) -> Result<BoardLayout, FenErr> {
    FenParser::parse_fen(fen)
}
