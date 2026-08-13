use logos::{Lexer, Logos, Skip};
use std::{
    error::Error,
    fmt::{Debug, Display},
    ops::Range,
};

use crate::acn_parser::{parse_algebraic_notation, PieceMove};

pub struct ParsedGame<'pgn> {
    pub tag_pairs: Vec<(&'pgn str, &'pgn str)>,
    pub moves: Vec<PieceMove>,
    pub result: GameResult,
}

impl<'pgn> Debug for ParsedGame<'pgn> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParsedGame")
            .field(
                "tag_pairs",
                &self
                    .tag_pairs
                    .iter()
                    .map(|tag_pair| (tag_pair.0.to_string(), tag_pair.1.to_string()))
                    .collect::<Vec<_>>(),
            )
            .field(
                "moves",
                &self
                    .moves
                    .iter()
                    .map(|move_| move_.to_string())
                    .collect::<Vec<_>>(),
            )
            .field("result", &format!("{:?}", self.result))
            .finish()
    }
}

impl<'pgn> ParsedGame<'pgn> {
    pub fn new(
        tag_pairs: Vec<(&'pgn str, &'pgn str)>,
        moves: Vec<PieceMove>,
        result: GameResult,
    ) -> Option<Self> {
        Some(Self {
            tag_pairs,
            moves,
            result,
        })
    }
}

impl<'pgn> ToString for &ParsedGame<'pgn> {
    fn to_string(&self) -> String {
        let mut result = String::new();

        for (i, tag_pair) in self.tag_pairs.iter().enumerate() {
            if i != 0 {
                result.push('\n');
            }

            result.push('[');
            result.push_str(&tag_pair.0);
            result.push(' ');

            result.push('"');
            result.push_str(&tag_pair.1);
            result.push('"');

            result.push(']');
        }

        result.push('\n');
        result.push('\n');

        let mut moves_iter = self.moves.iter();
        let mut move_num = 0;
        loop {
            let white_move = match moves_iter.next() {
                None => break,
                Some(move_) => move_,
            };

            if move_num != 0 {
                result.push(' ');
            }
            move_num += 1;
            result.push_str(&move_num.to_string());
            result.push('.');
            result.push(' ');
            result.push_str(&white_move.to_string());

            let black_move = match moves_iter.next() {
                None => break,
                Some(move_) => move_,
            };

            result.push(' ');
            result.push_str(&black_move.to_string());
        }

        result.push(' ');
        result.push_str(self.result.as_ref());
        result
    }
}

#[derive(Debug)]
pub enum GameResult {
    WhiteWin,
    BlackWin,
    Draw,
    Inconclusive,
}

impl AsRef<str> for GameResult {
    fn as_ref(&self) -> &str {
        match self {
            GameResult::WhiteWin => "1-0",
            GameResult::BlackWin => "0-1",
            GameResult::Draw => "1/2-1/2",
            GameResult::Inconclusive => "*",
        }
    }
}

#[derive(Debug)]
pub enum PgnErr<'pgn> {
    UnexpectedCharacter(PgnCharErr),
    Token(PgnTokenErr<'pgn>),
    InvalidTagName { span: Range<usize>, tag: &'pgn str },
    InvalidAlgebraicChessNotation { span: Span, value: &'pgn str },
}

pub struct PgnCharErr {
    location: Location,
}

impl PgnCharErr {
    pub fn location(&self) -> &Location {
        &self.location
    }
}

impl Debug for PgnCharErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Just defer to the Display impl
        write!(f, "{}", self)
    }
}

impl Display for PgnCharErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Found unexpected character at line {}, column {}, byte {}.",
            self.location.line, self.location.col, self.location.byte_index
        )
    }
}

pub struct PgnTokenErr<'pgn> {
    expected: Vec<PgnTokenKind<'pgn>>,
    not_expected: Vec<PgnTokenKind<'pgn>>,
    found: Option<PgnTokenKind<'pgn>>,
    found_span: Option<Span>,
}

impl<'pgn> PgnTokenErr<'pgn> {
    pub fn found(&self) -> Option<&PgnTokenKind<'pgn>> {
        self.found.as_ref()
    }

    pub fn found_span(&self) -> Option<&Span> {
        self.found_span.as_ref()
    }
}

impl<'pgn> Debug for PgnTokenErr<'pgn> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Just defer to the Display impl
        write!(f, "{}", self)
    }
}

impl<'pgn> Display for PgnTokenErr<'pgn> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let found_text = match &self.found {
            None => "EOF".to_string(),
            Some(token) => format!("{:?}", token),
        };

        if !self.not_expected.is_empty() {
            writeln!(
                f,
                "Did not expect any of {:?}, but found {:?}",
                self.not_expected, found_text
            )
        } else {
            writeln!(
                f,
                "Expected one of {:?}, but found {:?}",
                self.expected, found_text
            )
        }
    }
}

impl<'pgn> Error for PgnTokenErr<'pgn> {}

pub(crate) struct PgnParser<'pgn> {
    lookahead: Option<Result<PgnTokenKind<'pgn>, ()>>,
    tokenizer: Lexer<'pgn, PgnTokenKind<'pgn>>,
}

impl<'pgn> PgnParser<'pgn> {
    pub(crate) fn parse_pgn(source: &'pgn str) -> Result<Vec<ParsedGame<'pgn>>, PgnErr<'pgn>> {
        let mut parser = Self {
            lookahead: None,
            tokenizer: PgnTokenKind::lexer_with_extras(source, Extras::default()),
        };

        let mut games = Vec::new();
        loop {
            let tag_pairs = parser.match_tag_pairs()?;
            let move_text = parser.match_movetext()?;

            if tag_pairs.is_empty() && move_text.is_none() {
                break;
            }

            games.push(match move_text {
                None => ParsedGame {
                    tag_pairs,
                    result: GameResult::Inconclusive,
                    moves: Vec::with_capacity(0),
                },
                Some((result, moves)) => ParsedGame {
                    tag_pairs,
                    result,
                    moves,
                },
            });
        }

        return Ok(games);
    }

    fn match_tag_pairs(&mut self) -> Result<Vec<(&'pgn str, &'pgn str)>, PgnErr<'pgn>> {
        let mut result = Vec::new();

        while self
            .match_token_if(|token| matches!(token, PgnTokenKind::LeftSquareBracket))
            .is_some()
        {
            let symbol = self
                .match_token_or_err(PgnTokenKind::Symbol(Default::default()), |token| {
                    matches!(token, PgnTokenKind::Symbol(_))
                })?;
            let span = self.tokenizer.span();
            let string = self
                .match_token_or_err(PgnTokenKind::String(Default::default()), |token| {
                    matches!(token, PgnTokenKind::String(_))
                })?;
            self.match_token_or_err(PgnTokenKind::RightSquareBracket, |token| {
                matches!(token, PgnTokenKind::RightSquareBracket)
            })?;

            let symbol_string = if let PgnTokenKind::Symbol(string) = symbol {
                string
            } else {
                unreachable!()
            };

            let value_string = if let PgnTokenKind::String(bytes) = string {
                bytes
            } else {
                unreachable!()
            };

            if symbol_string
                .chars()
                .any(|ch: char| ch != '_' && !ch.is_ascii_alphanumeric())
            {
                return Err(PgnErr::InvalidTagName {
                    span,
                    tag: symbol_string,
                });
            }

            result.push((symbol_string, value_string));
        }

        return Ok(result);
    }

    fn match_movetext(&mut self) -> Result<Option<(GameResult, Vec<PieceMove>)>, PgnErr<'pgn>> {
        let mut moves = Vec::new();
        loop {
            // move numbers are optional in the import spec.
            while self
                .match_token_if(|token| matches!(token, PgnTokenKind::Comment(_)))
                .is_some()
            {}
            self.match_token_if(|token| matches!(token, PgnTokenKind::Integer(_)));
            while self
                .match_token_if(|token| matches!(token, PgnTokenKind::Comment(_)))
                .is_some()
            {}
            while self
                .match_token_if(|token| matches!(token, PgnTokenKind::Period))
                .is_some()
            {}

            while self
                .match_token_if(|token| matches!(token, PgnTokenKind::Comment(_)))
                .is_some()
            {}

            let white_move_token = if moves.is_empty() {
                match self.match_token_if(|token| {
                    matches!(
                        token,
                        PgnTokenKind::Symbol(_) | PgnTokenKind::GameTermination(_)
                    )
                }) {
                    None => return Ok(None),
                    Some(token) => token,
                }
            } else {
                self.match_token_or_err(PgnTokenKind::Symbol(Default::default()), |token| {
                    matches!(
                        token,
                        PgnTokenKind::Symbol(_) | PgnTokenKind::GameTermination(_)
                    )
                })?
            };

            match white_move_token {
                PgnTokenKind::GameTermination(termination) => match termination {
                    "1-0" => {
                        return Ok(Some((GameResult::WhiteWin, moves)));
                    }
                    "0-1" => {
                        return Ok(Some((GameResult::BlackWin, moves)));
                    }
                    "1/2-1/2" | "0.5-0.5" => {
                        return Ok(Some((GameResult::Draw, moves)));
                    }
                    "*" => {
                        return Ok(Some((GameResult::Inconclusive, moves)));
                    }
                    _ => {
                        unreachable!("logos grammar does not allow this form")
                    }
                },
                PgnTokenKind::Symbol(symbol) => match parse_algebraic_notation(&symbol.trim()) {
                    None => Err(PgnErr::InvalidAlgebraicChessNotation {
                        span: self.span(),
                        value: symbol,
                    })?,
                    Some(move_spec) => {
                        moves.push(move_spec);
                    }
                },
                _ => unreachable!(),
            }

            self.match_token_if(|token| matches!(token, PgnTokenKind::Comment(_)));
            match self.match_token_if(|token| {
                matches!(
                    token,
                    PgnTokenKind::Symbol(_) | PgnTokenKind::GameTermination(_)
                )
            }) {
                None => {}
                Some(PgnTokenKind::GameTermination(termination)) => match termination {
                    "1-0" => {
                        return Ok(Some((GameResult::WhiteWin, moves)));
                    }
                    "0-1" => {
                        return Ok(Some((GameResult::BlackWin, moves)));
                    }
                    "1/2-1/2" | "0.5-0.5" => {
                        return Ok(Some((GameResult::Draw, moves)));
                    }
                    "*" => {
                        return Ok(Some((GameResult::Inconclusive, moves)));
                    }
                    _ => unreachable!("logos grammar does not allow this form"),
                },
                Some(PgnTokenKind::Symbol(symbol)) => match parse_algebraic_notation(&symbol) {
                    None => Err(PgnErr::InvalidAlgebraicChessNotation {
                        span: self.span(),
                        value: symbol,
                    })?,
                    Some(move_spec) => {
                        moves.push(move_spec);
                    }
                },
                _ => unreachable!(),
            };

            // TODO: RAV
        }
    }

    fn match_token_or_err<'s, F>(
        &'s mut self,
        expected: PgnTokenKind<'pgn>,
        f: F,
    ) -> Result<PgnTokenKind<'pgn>, PgnErr<'pgn>>
    where
        F: FnOnce(&PgnTokenKind) -> bool,
    {
        match self.match_token_if(f) {
            None => Err(self.get_next_err_or_expected_token(vec![expected])),
            Some(token) => Ok(token),
        }
    }

    fn span(&self) -> Span {
        let range = self.tokenizer.span();
        let line = self.tokenizer.extras.line;
        let column = self.tokenizer.extras.column;

        Span {
            start: Location {
                line,
                col: column - (range.end - range.start),
                byte_index: range.start,
            },
            end: Location {
                line,
                col: column,
                byte_index: range.end,
            },
        }
    }

    fn get_next_err_or_expected_token<'s>(
        &'s mut self,
        expected_tokens: Vec<PgnTokenKind<'pgn>>,
    ) -> PgnErr<'pgn> {
        if let Some(result) = self.tokenizer.next() {
            match result {
                Err(()) => {
                    return PgnErr::UnexpectedCharacter(PgnCharErr {
                        location: self.span().start,
                    });
                }
                Ok(token) => {
                    return PgnErr::Token(PgnTokenErr {
                        expected: expected_tokens,
                        not_expected: Vec::new(),
                        found: Some(token),
                        found_span: Some(self.span()),
                    });
                }
            }
        }

        return PgnErr::Token(PgnTokenErr {
            expected: expected_tokens,
            not_expected: Vec::new(),
            found: None,
            found_span: None,
        });
    }

    fn match_token_if<'s, F>(&'s mut self, f: F) -> Option<PgnTokenKind<'pgn>>
    where
        F: FnOnce(&PgnTokenKind) -> bool,
    {
        match self.lookahead.take().or_else(|| self.tokenizer.next()) {
            None => None,
            Some(token) => match token {
                Err(_) => return None,
                Ok(token) => {
                    if f(&token) {
                        return Some(token);
                    } else {
                        self.lookahead = Some(Ok(token));
                        return None;
                    }
                }
            },
        }
    }
}

#[derive(Default)]
pub struct Extras {
    line: usize,
    column: usize,
}

/// Consult pgn_spec.html.
///
/// Based on the specification, the grammar for PGN is as follows:
/// File -> <br/>
///     (Line)* <br/>
/// Line -> <br/>
///     Token* <br/>
///     EscapedLine <br/>
/// EscapedLine -> <br/>
///     '%' any character except newline '\n' <br/>
/// Comment -> <br/>
///     ';' Any character except newline <br/>
///     '{' Any character except right brace '}' <br/>
///     # comments do not nest <br/>
/// Integer -> [0-9]+
/// LeftSquareBracket -> '['
/// RightSquareBracket -> ']'
/// LeftParen -> '('
/// RightParen -> ')'
/// LeftAngleBracket -> '<'
/// RightAngleBracket -> '>'
/// NAG (Numeric Annotation Glyph) -> '$' [0-9]+
/// String -> '"' PrintingCharacters '"' <br/>
/// Symbol -> [a-zA-Z0-9] followed by [a-zA-Z0-9_+#=:-]*
///     # maximum 255 characters in length
/// Whitespace -> <br/>
///     byte codes decimal 11 through decimal 13 as well as decimal 15 and 20 <br/>
///     # horizontal and vertical tab are not allowed in export format
/// PrintingCharacters -> <br/>
///     byte code decimal 32 through decimal 126 (inclusive) <br/>
///     byte code decimal 160 through decimal 191 (inclusive) # these codes are discouraged, but allowed in the spec <br/>
///     byte code decimal 192 through decimal 255 (inclusive) # allowed, but should be represented by '?' if the software cannot handle rendering of these characters <br/>

#[derive(Logos, Debug, PartialEq)]
#[logos(extras = Extras)]
#[logos(skip(r"[ \t\r]", skip_callback))]
#[logos(skip(r#"\n"#, newline_callback))]
pub enum PgnTokenKind<'s> {
    #[token("[", advance_token)]
    LeftSquareBracket,
    #[token("]", advance_token)]
    RightSquareBracket,
    #[token("<", advance_token)]
    LeftAngleBracket,
    #[token(">", advance_token)]
    RightAngleBracket,
    #[token("(", advance_token)]
    LeftParen,
    #[token(")", advance_token)]
    RightParen,
    #[token(".", advance_token)]
    Period,
    /// Numeric Annotation Glyph
    #[regex(r#"\$\d+"#, nag_callback)]
    NAG(&'s str),
    #[regex(r#""(?:[^"\\]|\\["\\])*?""#, get_slice_callback)]
    String(&'s str),
    #[regex(r#"\d+"#, |lex| lex.slice(), priority = 3)]
    Integer(&'s str),
    #[regex(r#"[a-zA-Z0-9][a-zA-Z0-9_+#=:\-]+"#, get_slice_callback, priority = 2)]
    Symbol(&'s str),
    #[token("1-0", get_slice_callback)]
    #[token("0-1", get_slice_callback)]
    #[token("*", get_slice_callback)]
    #[token("1/2-1/2", get_slice_callback)]
    #[token("0.5-0.5", get_slice_callback)]
    GameTermination(&'s str),
    #[regex(r#"(?:;.*?\n)|(?:\{[^\}]*\})"#, get_slice_callback)]
    Comment(&'s str),
    #[regex(r#"%.*?\n"#, get_slice_callback)]
    EscapedLine(&'s str),
}

fn advance_token<'pgn>(lex: &mut Lexer<'pgn, PgnTokenKind<'pgn>>) {
    lex.extras.column += lex.slice().len();
}

fn newline_callback<'pgn>(lex: &mut Lexer<'pgn, PgnTokenKind<'pgn>>) -> Skip {
    lex.extras.line += 1;
    lex.extras.column = 0;
    Skip
}

fn skip_callback<'pgn>(lex: &mut Lexer<'pgn, PgnTokenKind<'pgn>>) -> Skip {
    lex.extras.column += lex.slice().len();
    Skip
}

fn nag_callback<'pgn>(lex: &mut Lexer<'pgn, PgnTokenKind<'pgn>>) -> &'pgn str {
    lex.extras.column += lex.slice().len();
    &lex.slice()['$'.len_utf8()..]
}

fn get_slice_callback<'pgn>(lex: &mut Lexer<'pgn, PgnTokenKind<'pgn>>) -> &'pgn str {
    lex.slice()
}

#[derive(Debug)]
pub struct Span {
    start: Location,
    end: Location,
}

impl Into<Range<usize>> for &Span {
    fn into(self) -> Range<usize> {
        self.start.byte_index()..self.end.byte_index()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Location {
    line: usize,
    col: usize,
    byte_index: usize,
}

impl Location {
    pub fn line(&self) -> usize {
        self.line
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn byte_index(&self) -> usize {
        self.byte_index
    }
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use crate::pgn_parser::PgnTokenKind;

    use super::PgnParser;

    #[test]
    fn parses_empty_pgn() {
        PgnParser::parse_pgn("").unwrap();
    }

    #[test]
    fn parses_full_game() {
        let pgn = r#"
        1. e4 e5 2. Nf3 d6 3. d4 Bg4 {This is a weak move 
        already - Fischer} 4. dxe5 Bxf3 5. Qxf3 dxe5 6. Bc4 Nf6 7. Qb3 Qe7
        8. Nc3 c6 9. Bg5 {Black is in a zugzwang-like position
        here. He can't develop the queen's knight because the pawn
        is hanging, the bishop is blocked because of the 
        queen.-Fischer} b5 10. Nxb5 cxb5 11. Bxb5+ Nbd7 12. O-O-O Rd8
        13. Rxd7 Rxd7 14. Rd1 Qe6 15. Bxd7+ Nxd7 16. Qb8+ Nxb8 17. Rd8# 1-0
        "#;

        let parsed = PgnParser::parse_pgn(pgn).unwrap();
        println!("{:#?}", parsed);
    }

    #[test]
    fn parses_real_pgn() {
        let pgn = include_str!("../Bucharest2023.pgn");

        let mut tokens = Vec::new();
        let mut errs = Vec::new();
        for token in PgnTokenKind::lexer(pgn) {
            match token {
                Err(err) => errs.push(err),
                Ok(token) => tokens.push(token),
            }
        }

        for token in tokens {
            println!("{:?}", token);
        }
        for err in errs {
            println!("{:#?}", err);
        }

        for game in PgnParser::parse_pgn(pgn).unwrap() {
            println!("{:#?}", game);
        }
    }
}

#[test]
fn test() {
    use std::fs::OpenOptions;
    use std::io::Read;
    let mut buf = String::new();
    OpenOptions::new()
        .read(true)
        .write(false)
        .open("C:/Users/ad4mb/Downloads/lichess_db_broadcast_2026-07.pgn")
        .map(|mut file| file.read_to_string(&mut buf))
        .flatten()
        .unwrap();

    let games = PgnParser::parse_pgn(&buf).unwrap();
    for game in games {
        println!("{:#?}", game);
    }
}
