mod pgn_parser;
use pgn_parser::PgnParser;
pub use pgn_parser::{ParsedGame, PgnErr};

use fen_parser::FenParser;
pub use fen_parser::{BoardLayout, FenErr};

mod acn_parser;
mod fen_parser;

pub use acn_parser::{parse_algebraic_notation, PieceMove, PieceMoveKind, NormalMove, Check};

pub fn parse_pgn(pgn: &[u8]) -> Result<Vec<ParsedGame>, PgnErr> {
    PgnParser::parse_pgn(pgn)
}

pub fn parse_fen(fen: &str) -> Result<BoardLayout, FenErr> {
    FenParser::parse_fen(fen)
}
