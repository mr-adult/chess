use iso_8859_1_encoder::Iso8859String;
use std::{
    error::Error,
    fmt::{Debug, Display},
    iter::{Enumerate, Peekable},
    slice::Iter,
};

use crate::acn_parser::{parse_algebraic_notation, PieceMove};

pub struct ParsedGame {
    pub tag_pairs: Vec<(String, String)>,
    pub moves: Vec<PieceMove>,
    pub result: GameResult,
}

impl Debug for ParsedGame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = "ParsedGame {\n".to_string();
        result.push_str("\ttag_pairs: [");
        for tag_pair in self.tag_pairs.iter() {
            result.push_str("\n\t\t");
            result.push_str(&tag_pair.0);
            result.push_str(": ");
            result.push_str(&tag_pair.1);
        }
        if !self.tag_pairs.is_empty() {
            result.push_str("\n\t");
        }
        result.push_str("],\n");

        result.push_str("\tmoves: [");
        for move_ in self.moves.iter() {
            result.push_str("\n\t\t");
            result.push_str(&format!("{:?}", move_));
        }
        if !self.moves.is_empty() {
            result.push('\n');
        }
        result.push_str("\t],\n");

        result.push_str("\tresult: ");
        result.push_str(&format!("{:?}", self.result));

        write!(f, "{}", result)
    }
}

impl ParsedGame {}

#[derive(Debug)]
pub enum GameResult {
    WhiteWin,
    BlackWin,
    Draw,
    Inconclusive,
}

#[derive(Debug)]
pub enum PgnErr {
    Byte(PgnByteErr),
    Token(PgnTokenErr),
    InvalidTagName(String),
    InvalidAlgebraicChessNotation(String),
}

pub struct PgnByteErr {
    expected: Vec<char>,
    not_expected: Vec<char>,
    found: Option<u8>,
    byte_index: Option<usize>,
}

impl Debug for PgnByteErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Just defer to the Display impl
        write!(f, "{}", self)
    }
}

impl Display for PgnByteErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let byte_index_text = match self.byte_index {
            None => "EOF".to_string(),
            Some(index) => index.to_string(),
        };

        let found_text = match self.found {
            None => "EOF".to_string(),
            Some(byte) => Iso8859String::from_bytes(vec![byte]).to_string(),
        };

        if !self.not_expected.is_empty() {
            writeln!(
                f,
                "(byte {}) Did not expect any of {:?}, but found {}",
                byte_index_text, self.not_expected, found_text
            )
        } else {
            writeln!(
                f,
                "(byte {}) Expected one of {:?}, but found {:?}",
                byte_index_text, self.expected, found_text
            )
        }
    }
}

pub struct PgnTokenErr {
    expected: Vec<PgnTokenKind>,
    not_expected: Vec<PgnTokenKind>,
    found: Option<PgnToken>,
}

impl Debug for PgnTokenErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Just defer to the Display impl
        write!(f, "{}", self)
    }
}

impl Display for PgnTokenErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let found_text = match &self.found {
            None => "EOF".to_string(),
            Some(token) => format!("{:?}", token.kind()),
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

impl Error for PgnTokenErr {}

pub(crate) struct PgnParser<'pgn> {
    tokenizer: Peekable<PgnTokenizer<'pgn>>,
}

impl<'pgn> PgnParser<'pgn> {
    pub(crate) fn parse_pgn(source: &'pgn [u8]) -> Result<Vec<ParsedGame>, PgnErr> {
        let mut parser = Self {
            tokenizer: PgnTokenizer::new(source).peekable(),
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

    fn match_tag_pairs(&mut self) -> Result<Vec<(String, String)>, PgnErr> {
        let mut result = Vec::new();

        while self.match_token(PgnTokenKind::LeftSquareBracket).is_some() {
            let symbol = self.match_token_or_err(PgnTokenKind::Symbol)?;
            let string = self.match_token_or_err(PgnTokenKind::String)?;
            self.match_token_or_err(PgnTokenKind::RightSquareBracket)?;

            let symbol_string = if let PgnToken::Symbol(string) = symbol {
                string
            } else {
                unreachable!()
            };

            let value_string = if let PgnToken::String(bytes) = string {
                bytes
            } else {
                unreachable!()
            };

            if symbol_string
                .chars()
                .any(|ch| ch != '_' && !ch.is_ascii_alphanumeric())
            {
                return Err(PgnErr::InvalidTagName(symbol_string));
            }

            result.push((symbol_string, Iso8859String::from_bytes(value_string).to_string()));
        }

        return Ok(result);
    }

    fn match_movetext(&mut self) -> Result<Option<(GameResult, Vec<PieceMove>)>, PgnErr> {
        let mut moves = Vec::new();
        loop {
            // move numbers are optional in the import spec.
            while self.match_token(PgnTokenKind::Comment).is_some() {}
            self.match_token(PgnTokenKind::Integer);
            while self.match_token(PgnTokenKind::Comment).is_some() {}
            while self.match_token(PgnTokenKind::Period).is_some() {}

            while self.match_token(PgnTokenKind::Comment).is_some() {}

            let white_move_token = if moves.is_empty() {
                match self.match_token(PgnTokenKind::Symbol) {
                    None => return Ok(None),
                    Some(token) => token,
                }
            } else {
                self.match_token_or_err(PgnTokenKind::Symbol)?
            };

            if let PgnToken::Symbol(symbol) = white_move_token {
                match symbol.as_str() {
                    "1-0" => {
                        return Ok(Some((GameResult::WhiteWin, moves)));
                    }
                    "0-1" => {
                        return Ok(Some((GameResult::BlackWin, moves)));
                    }
                    "1/2-1/2" => {
                        return Ok(Some((GameResult::Draw, moves)));
                    }
                    "*" => {
                        return Ok(Some((GameResult::Inconclusive, moves)));
                    }
                    _ => {}
                }

                match parse_algebraic_notation(&symbol.trim()) {
                    None => Err(PgnErr::InvalidAlgebraicChessNotation(symbol))?,
                    Some(move_spec) => {
                        moves.push(move_spec);
                    }
                }
            } else {
                unreachable!();
            };

            self.match_token(PgnTokenKind::Comment);
            match self.match_token(PgnTokenKind::Symbol) {
                None => {}
                Some(PgnToken::Symbol(symbol)) => {
                    match symbol.as_str() {
                        "1-0" => {
                            return Ok(Some((GameResult::WhiteWin, moves)));
                        }
                        "0-1" => {
                            return Ok(Some((GameResult::BlackWin, moves)));
                        }
                        "1/2-1/2" => {
                            return Ok(Some((GameResult::Draw, moves)));
                        }
                        "*" => {
                            return Ok(Some((GameResult::Inconclusive, moves)));
                        }
                        _ => {}
                    }
                    match parse_algebraic_notation(&symbol) {
                        None => Err(PgnErr::InvalidAlgebraicChessNotation(symbol))?,
                        Some(move_spec) => {
                            moves.push(move_spec);
                        }
                    }
                }
                _ => unreachable!(),
            };

            // TODO: RAV
        }
    }

    fn match_token_or_err(&mut self, kind: PgnTokenKind) -> Result<PgnToken, PgnErr> {
        match self.match_token(kind) {
            None => Err(self.get_next_err_or_expected_token(vec![kind])),
            Some(token) => Ok(token),
        }
    }

    fn get_next_err_or_expected_token(&mut self, expected_tokens: Vec<PgnTokenKind>) -> PgnErr {
        if let Some(result) = self.tokenizer.next() {
            match result {
                Err(err) => {
                    return PgnErr::Byte(err);
                }
                Ok(token) => {
                    return PgnErr::Token(PgnTokenErr {
                        expected: expected_tokens,
                        not_expected: Vec::new(),
                        found: Some(token),
                    });
                }
            }
        }

        return PgnErr::Token(PgnTokenErr {
            expected: expected_tokens,
            not_expected: Vec::new(),
            found: None,
        });
    }

    fn match_token(&mut self, kind: PgnTokenKind) -> Option<PgnToken> {
        match self.tokenizer.peek() {
            None => None,
            Some(token) => match token {
                Err(_) => return None,
                Ok(token) => {
                    if token.kind() == kind {
                        return Some(
                            self.tokenizer
                                .next()
                                .expect("Next to be Some()")
                                .expect("Next to be Ok()"),
                        );
                    } else {
                        return None;
                    }
                }
            },
        }
    }
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
struct PgnTokenizer<'pgn> {
    bytes: Peekable<Enumerate<Iter<'pgn, u8>>>,
    errored: bool,
}

impl<'pgn> PgnTokenizer<'pgn> {
    fn new(source: &'pgn [u8]) -> Self {
        Self {
            bytes: source.iter().enumerate().peekable(),
            errored: false,
        }
    }

    /// Returns whether or not the last character matched was a '\n'
    fn match_whitespace(&mut self) -> bool {
        let mut last_is_newline = false;
        self.match_byte_while_throwaway(&mut |byte| {
            if Self::is_whitespace(byte) {
                if byte == b'\n' {
                    last_is_newline = true;
                } else {
                    last_is_newline = false;
                }
                return true;
            }
            return false;
        });
        return last_is_newline;
    }

    fn is_whitespace(byte: u8) -> bool {
        match byte {
                | b'\n'
                | b'\t'
                | 0x000B // vertical tab
                | b'\r'
                | b' ' => true,
                _ => false,
        }
    }

    fn is_decimal_digit(byte: u8) -> bool {
        matches!(byte, b'0'..=b'9')
    }

    fn is_symbol_continuation(byte: u8) -> bool {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'+' | b'#' | b'=' | b':' | b'-' => {
                true
            }
            _ => false,
        }
    }

    fn is_printing_char(byte: u8) -> bool {
        match byte {
            // need to skip 34, which is the quote character
            32 | 33 | 35..=126 | 160..=191 | 192..=255 => true,
            _ => Self::is_whitespace(byte),
        }
    }

    fn match_byte_while_throwaway<F: FnMut(u8) -> bool>(&mut self, func: &mut F) {
        while let Some(_) = self.match_byte_if(func) {}
    }

    fn match_byte_while<F: FnMut(u8) -> bool>(
        &mut self,
        func: &mut F,
        vec_to_append_to: &mut Vec<u8>,
    ) {
        while let Some(byte) = self.match_byte_if(func) {
            vec_to_append_to.push(byte);
        }
    }

    fn match_byte(&mut self, byte: u8) -> bool {
        self.match_byte_if(&mut |other_byte| byte == other_byte)
            .is_some()
    }

    fn match_byte_if<F: FnMut(u8) -> bool>(&mut self, func: &mut F) -> Option<u8> {
        match self.bytes.peek() {
            None => return None,
            Some((_, byte)) => {
                if func(**byte) {
                    return Some(
                        *self
                            .bytes
                            .next()
                            .expect("Should always be Some() since peek() returned Some()")
                            .1,
                    );
                } else {
                    return None;
                }
            }
        }
    }
}

impl<'pgn> Iterator for PgnTokenizer<'pgn> {
    type Item = Result<PgnToken, PgnByteErr>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            return None;
        }

        if self.match_whitespace() {
            if let Some((_, b'%')) = self.bytes.peek() {
                self.bytes.next();
                let mut result = Vec::new();
                self.match_byte_while(&mut |byte| byte != b'\n', &mut result);
                return Some(Ok(PgnToken::EscapedLine(result)));
            }
        }

        if let Some((index, byte)) = self.bytes.next() {
            match *byte {
                // Do the self-closing tokens first since they have simpler handling.
                b'{' => {
                    let mut comment = Vec::new();

                    self.match_byte_while(&mut |byte| byte != b'}', &mut comment);
                    if self.bytes.peek().is_none() {
                        return Some(Err(PgnByteErr {
                            expected: vec!['}'],
                            not_expected: Vec::new(),
                            found: None,
                            byte_index: None,
                        }));
                    }

                    self.match_byte(b'}');
                    return Some(Ok(PgnToken::Comment(comment)));
                }
                b'[' => Some(Ok(PgnToken::LeftSquareBracket)),
                b']' => Some(Ok(PgnToken::RightSquareBracket)),
                b'<' => Some(Ok(PgnToken::LeftAngleBracket)),
                b'>' => Some(Ok(PgnToken::RightAngleBracket)),
                b'(' => Some(Ok(PgnToken::LeftParen)),
                b')' => Some(Ok(PgnToken::RightParen)),
                b'.' => Some(Ok(PgnToken::Period)),
                b'*' => Some(Ok(PgnToken::Asterisk)),
                b';' => {
                    let mut comment = Vec::new();
                    let mut prev_was_carriage_return = false;
                    self.match_byte_while(
                        &mut |byte| match byte {
                            b'\r' => {
                                prev_was_carriage_return = true;
                                return true;
                            }
                            b'\n' => {
                                return false;
                            }
                            _ => {
                                prev_was_carriage_return = false;
                                return true;
                            }
                        },
                        &mut comment,
                    );
                    // handle \r\n's for the caller
                    if prev_was_carriage_return {
                        comment.pop();
                    }
                    self.match_byte(b'\n');
                    return Some(Ok(PgnToken::Comment(comment)));
                }
                b'$' => {
                    let mut result = Vec::new();

                    if let Some(byte) = self.match_byte_if(&mut Self::is_decimal_digit) {
                        result.push(byte);
                    } else {
                        return Some(Err(PgnByteErr {
                            expected: vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
                            not_expected: Vec::new(),
                            found: match self.bytes.peek() {
                                None => None,
                                Some((_, ch)) => Some(**ch),
                            },
                            byte_index: match self.bytes.peek() {
                                None => None,
                                Some((index, _)) => Some(*index),
                            },
                        }));
                    }

                    self.match_byte_while(&mut Self::is_decimal_digit, &mut result);

                    let string = unsafe { String::from_utf8_unchecked(result) };
                    let parsed = string.parse::<usize>().ok();

                    return Some(Ok(PgnToken::NAG(Integer {
                        raw: string,
                        parsed,
                    })));
                }
                b'"' => {
                    let mut string_content = Vec::new();
                    let mut previous_was_backslash = false;
                    self.match_byte_while(
                        &mut |byte| {
                            if byte == b'\\' {
                                previous_was_backslash = true;
                            } else {
                                previous_was_backslash = false;
                            }

                            Self::is_printing_char(byte) || (previous_was_backslash && byte == b'"')
                        },
                        &mut string_content,
                    );
                    if self.match_byte(b'"') {
                        return Some(Ok(PgnToken::String(string_content)));
                    } else {
                        return Some(Err(match self.bytes.next() {
                            None => PgnByteErr {
                                expected: vec!['"'],
                                not_expected: Vec::new(),
                                found: None,
                                byte_index: None,
                            },
                            Some((index, byte)) => PgnByteErr {
                                expected: vec!['"'],
                                not_expected: Vec::new(),
                                found: Some(*byte),
                                byte_index: Some(index),
                            },
                        }));
                    }
                }
                b'0'..=b'9' => {
                    let mut result = Vec::new();
                    result.push(*byte);
                    let matched_solidus = if *byte == b'1' && self.match_byte(b'/') {
                        result.push(b'/');
                        true
                    } else {
                        false
                    };
                    self.match_byte_while(&mut Self::is_decimal_digit, &mut result);

                    let len_before = result.len();
                    self.match_byte_while(&mut Self::is_symbol_continuation, &mut result);
                    if matched_solidus || len_before != result.len() {
                        if result.len() == 5
                            && result.last() == Some(&b'1')
                            && self.match_byte(b'/')
                        {
                            // allow 1/2-1/2 to be counted as a symbol
                            result.push(b'/');
                            if self.match_byte(b'2') {
                                result.push(b'2');
                            }
                        } else {
                            for _ in 0..2 {
                                self.match_byte_if(&mut |byte| byte == b'!' || byte == b'?');
                            }
                        }

                        return Some(Ok(PgnToken::Symbol(unsafe {
                            String::from_utf8_unchecked(result)
                        })));
                    }

                    let string = unsafe { String::from_utf8_unchecked(result) };

                    let parsed = string.parse::<usize>().ok();
                    Some(Ok(PgnToken::Integer(Integer {
                        raw: string,
                        parsed,
                    })))
                }
                b'a'..=b'z' | b'A'..=b'Z' => {
                    let mut result = Vec::with_capacity(5);
                    result.push(*byte);
                    self.match_byte_while(&mut Self::is_symbol_continuation, &mut result);
                    for _ in 0..2 {
                        self.match_byte_if(&mut |byte| byte == b'!' || byte == b'?');
                    }

                    return Some(Ok(PgnToken::Symbol(unsafe {
                        String::from_utf8_unchecked(result)
                    })));
                }
                _ => {
                    return Some(Err(PgnByteErr {
                        expected: vec![
                            '{', '[', ']', '<', '>', '(', ')', '.', '*', '$', '"', '0', '1', '2',
                            '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
                            'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
                            'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
                            'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W',
                            'X', 'Y', 'Z',
                        ],
                        not_expected: Vec::new(),
                        found: Some(*byte),
                        byte_index: Some(index),
                    }));
                }
            }
        } else {
            return None;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PgnTokenKind {
    LeftSquareBracket,
    RightSquareBracket,
    LeftAngleBracket,
    RightAngleBracket,
    LeftParen,
    RightParen,
    Period,
    Asterisk,
    NAG,
    String,
    Integer,
    Symbol,
    Comment,
    EscapedLine,
}

#[derive(Debug)]
enum PgnToken {
    // Self-closing tokens first
    /// [
    LeftSquareBracket,
    /// ]
    RightSquareBracket,
    /// <
    LeftAngleBracket,
    /// >
    RightAngleBracket,
    /// (
    LeftParen,
    /// )
    RightParen,
    /// .
    Period,
    /// *
    Asterisk,
    /// Numeric Annotation Glyph
    #[allow(unused)]
    NAG(Integer),
    #[allow(unused)]
    String(Vec<u8>),
    #[allow(unused)]
    Integer(Integer),
    #[allow(unused)]
    Symbol(String),
    #[allow(unused)]
    Comment(Vec<u8>),
    #[allow(unused)]
    EscapedLine(Vec<u8>),
}

impl PgnToken {
    fn kind(&self) -> PgnTokenKind {
        match self {
            PgnToken::LeftSquareBracket => PgnTokenKind::LeftSquareBracket,
            PgnToken::RightSquareBracket => PgnTokenKind::RightSquareBracket,
            PgnToken::LeftAngleBracket => PgnTokenKind::LeftAngleBracket,
            PgnToken::RightAngleBracket => PgnTokenKind::RightAngleBracket,
            PgnToken::LeftParen => PgnTokenKind::LeftParen,
            PgnToken::RightParen => PgnTokenKind::RightParen,
            PgnToken::Period => PgnTokenKind::Period,
            PgnToken::Asterisk => PgnTokenKind::Asterisk,
            PgnToken::NAG(_) => PgnTokenKind::NAG,
            PgnToken::String(_) => PgnTokenKind::String,
            PgnToken::Integer(_) => PgnTokenKind::Integer,
            PgnToken::Symbol(_) => PgnTokenKind::Symbol,
            PgnToken::Comment(_) => PgnTokenKind::Comment,
            PgnToken::EscapedLine(_) => PgnTokenKind::EscapedLine,
        }
    }
}

#[derive(Debug)]
struct Integer {
    #[allow(unused)]
    raw: String,
    #[allow(unused)]
    parsed: Option<usize>,
}

#[cfg(test)]
mod tests {
    use crate::pgn_parser::PgnTokenizer;

    use super::PgnParser;

    #[test]
    fn parses_empty_pgn() {
        PgnParser::parse_pgn(b"").unwrap();
    }

    #[test]
    fn parses_full_game() {
        let pgn = br#"
        1. e4 e5 2. Nf3 d6 3. d4 Bg4 {This is a weak move 
        already - Fischer} 4. dxe5 Bxf3 5. Qxf3 dxe5 6. Bc4 Nf6 7. Qb3 Qe7
        8. Nc3 c6 9. Bg5 {Black is in a zugzwang-like position
        here. He can't develop the queen's knight because the pawn
        is hanging, the bishop is blocked because of the 
        queen.-Fischer} b5 10. Nxb5 cxb5 11. Bxb5+ Nbd7 12. O-O-O Rd8
        13. Rxd7 Rxd7 14. Rd1 Qe6 15. Bxd7+ Nxd7 16. Qb8+ Nxb8 17. Rd8# 1-0
        "#;

        let parsed = PgnParser::parse_pgn(pgn).unwrap();
        println!("{:?}", parsed);
    }

    #[test]
    fn parses_real_pgn() {
        let pgn = include_bytes!("../Bucharest2023.pgn");

        let mut tokens = Vec::new();
        let mut errs = Vec::new();
        for token in PgnTokenizer::new(pgn) {
            match token {
                Err(err) => errs.push(err),
                Ok(token) => tokens.push(token),
            }
        }

        for token in tokens {
            println!("{:?}", token);
        }
        for err in errs {
            println!("{:?}", err);
        }

        for game in PgnParser::parse_pgn(pgn).unwrap() {
            println!("{:?}", game);
        }
    }
}
