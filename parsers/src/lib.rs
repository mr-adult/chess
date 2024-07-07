mod pgn_parser;
use pgn_parser::PgnParser;
pub use pgn_parser::{ParsedGame, PgnErr};
mod fen;
use fen::FenParser;
pub use fen::{BoardLayout, FenErr, PieceLocations};
mod acn_parser;
pub use acn_parser::{parse_algebraic_notation, Check, NormalMove, PieceMove, PieceMoveKind};

pub fn parse_pgn(pgn: &[u8]) -> Result<Vec<ParsedGame>, PgnErr> {
    PgnParser::parse_pgn(pgn)
}

pub fn parse_fen(fen: &str) -> Result<BoardLayout, FenErr> {
    FenParser::parse_fen(fen)
}
