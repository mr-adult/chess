use iso_8859_1_encoder::{Iso8859String, Iso8859TranscodeErr};
use std::{
    error::Error,
    fmt::{Debug, Display},
    iter::{Cloned, Enumerate, Peekable},
    ops::Range,
    slice::Iter,
};

use crate::acn_parser::{parse_algebraic_notation, PieceMove};

pub struct ParsedGame {
    pub tag_pairs: Vec<(Iso8859String, Iso8859String)>,
    pub moves: Vec<PieceMove>,
    pub result: GameResult,
}

impl Debug for ParsedGame {
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

impl ParsedGame {
    pub fn new(
        tag_pairs: Vec<(String, String)>,
        moves: Vec<PieceMove>,
        result: GameResult,
    ) -> Option<Self> {
        // TODO: handle errors in the input
        let result_tags = tag_pairs
            .into_iter()
            .map(|tuple| {
                let result1: Iso8859String = (&tuple.0).try_into()?;
                let result2: Iso8859String = (&tuple.1).try_into()?;
                Ok((result1, result2))
            })
            .collect::<Result<Vec<_>, Iso8859TranscodeErr>>();

        let tag_pairs = match result_tags {
            Err(_) => return None,
            Ok(tags) => tags,
        };

        Some(Self {
            tag_pairs,
            moves,
            result,
        })
    }
}

impl Into<Iso8859String> for &ParsedGame {
    fn into(self) -> Iso8859String {
        let mut result = Vec::new();

        for (i, tag_pair) in self.tag_pairs.iter().enumerate() {
            if i != 0 {
                result.push(b'\n');
            }

            result.push(b'[');
            for byte in tag_pair.0.as_bytes() {
                result.push(*byte);
            }

            result.push(b' ');

            result.push(b'"');
            for byte in tag_pair.1.as_bytes() {
                result.push(*byte);
            }
            result.push(b'"');

            result.push(b']');
        }

        result.push(b'\n');
        result.push(b'\n');

        let mut moves_iter = self.moves.iter();
        let mut move_num = 0;
        loop {
            let white_move = match moves_iter.next() {
                None => break,
                Some(move_) => move_,
            };

            if move_num != 0 {
                result.push(b' ');
            }
            move_num += 1;
            for byte in move_num.to_string().as_bytes() {
                result.push(*byte);
            }
            result.push(b'.');
            result.push(b' ');
            for byte in white_move.to_string().as_bytes() {
                result.push(*byte);
            }

            let black_move = match moves_iter.next() {
                None => break,
                Some(move_) => move_,
            };

            result.push(b' ');
            for byte in black_move.to_string().as_bytes() {
                result.push(*byte);
            }
        }

        result.push(b' ');
        for byte in Iso8859String::try_from(self.result.as_ref())
            .unwrap()
            .as_bytes()
        {
            result.push(*byte);
        }

        Iso8859String::from_bytes(result)
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
pub enum PgnErr {
    Byte(PgnByteErr),
    Token(PgnTokenErr),
    InvalidTagName { span: Span, tag: String },
    InvalidAlgebraicChessNotation { span: Span, value: String },
}

pub struct PgnByteErr {
    expected: Vec<char>,
    not_expected: Vec<char>,
    found: Option<u8>,
    location: Location,
}

impl PgnByteErr {
    pub fn location(&self) -> &Location {
        &self.location
    }

    pub fn expected(&self) -> &[char] {
        &self.expected
    }
}

impl Debug for PgnByteErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Just defer to the Display impl
        write!(f, "{}", self)
    }
}

impl Display for PgnByteErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let byte_index_text = self.location.byte_index.to_string();

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

impl PgnTokenErr {
    pub fn found(&self) -> Option<&PgnToken> {
        self.found.as_ref()
    }
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

            let tag_pairs = tag_pairs
                .into_iter()
                .map(|(one, two)| {
                    (
                        Iso8859String::try_from(&one).unwrap(),
                        Iso8859String::try_from(&two).unwrap(),
                    )
                })
                .collect();

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

        while self
            .match_token_if(|token| matches!(token.kind(), PgnTokenKind::LeftSquareBracket))
            .is_some()
        {
            let symbol = self
                .match_token_or_err(PgnTokenKind::Symbol(Default::default()), |token| {
                    matches!(token.kind(), PgnTokenKind::Symbol(_))
                })?;
            let string = self
                .match_token_or_err(PgnTokenKind::String(Default::default()), |token| {
                    matches!(token.kind(), PgnTokenKind::String(_))
                })?;
            self.match_token_or_err(PgnTokenKind::RightSquareBracket, |token| {
                matches!(token.kind(), PgnTokenKind::RightSquareBracket)
            })?;

            let symbol_string = if let PgnTokenKind::Symbol(string) = symbol.kind {
                string
            } else {
                unreachable!()
            };

            let value_string = if let PgnTokenKind::String(bytes) = string.kind {
                bytes
            } else {
                unreachable!()
            };

            if symbol_string
                .chars()
                .any(|ch| ch != '_' && !ch.is_ascii_alphanumeric())
            {
                return Err(PgnErr::InvalidTagName {
                    span: symbol.span,
                    tag: symbol_string,
                });
            }

            result.push((
                symbol_string,
                Iso8859String::from_bytes(value_string).to_string(),
            ));
        }

        return Ok(result);
    }

    fn match_movetext(&mut self) -> Result<Option<(GameResult, Vec<PieceMove>)>, PgnErr> {
        let mut moves = Vec::new();
        loop {
            // move numbers are optional in the import spec.
            while self
                .match_token_if(|token| matches!(token.kind(), PgnTokenKind::Comment(_)))
                .is_some()
            {}
            self.match_token_if(|token| matches!(token.kind(), PgnTokenKind::Integer(_)));
            while self
                .match_token_if(|token| matches!(token.kind(), PgnTokenKind::Comment(_)))
                .is_some()
            {}
            while self
                .match_token_if(|token| matches!(token.kind(), PgnTokenKind::Period))
                .is_some()
            {}

            while self
                .match_token_if(|token| matches!(token.kind(), PgnTokenKind::Comment(_)))
                .is_some()
            {}

            let white_move_token = if moves.is_empty() {
                match self.match_token_if(|token| matches!(token.kind(), PgnTokenKind::Symbol(_))) {
                    None => return Ok(None),
                    Some(token) => token,
                }
            } else {
                self.match_token_or_err(PgnTokenKind::Symbol(Default::default()), |token| {
                    matches!(token.kind(), PgnTokenKind::Symbol(_))
                })?
            };

            if let PgnTokenKind::Symbol(symbol) = white_move_token.kind {
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
                    None => Err(PgnErr::InvalidAlgebraicChessNotation {
                        span: white_move_token.span,
                        value: symbol,
                    })?,
                    Some(move_spec) => {
                        moves.push(move_spec);
                    }
                }
            } else {
                unreachable!();
            };

            self.match_token_if(|token| matches!(token.kind(), PgnTokenKind::Comment(_)));
            match self.match_token_if(|token| matches!(token.kind(), PgnTokenKind::Symbol(_))) {
                None => {}
                Some(PgnToken {
                    kind: PgnTokenKind::Symbol(symbol),
                    span,
                }) => {
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
                        None => Err(PgnErr::InvalidAlgebraicChessNotation {
                            span,
                            value: symbol,
                        })?,
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

    fn match_token_or_err<F>(&mut self, expected: PgnTokenKind, f: F) -> Result<PgnToken, PgnErr>
    where
        F: FnOnce(&PgnToken) -> bool,
    {
        match self.match_token_if(f) {
            None => Err(self.get_next_err_or_expected_token(vec![expected])),
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

    fn match_token_if<F>(&mut self, f: F) -> Option<PgnToken>
    where
        F: FnOnce(&PgnToken) -> bool,
    {
        match self.tokenizer.peek() {
            None => None,
            Some(token) => match token {
                Err(_) => return None,
                Ok(token) => {
                    if f(token) {
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
    len: usize,
    bytes: Peekable<ByteLocations<Enumerate<Cloned<Iter<'pgn, u8>>>>>,
    line: usize,
    col: usize,
    errored: bool,
}

impl<'pgn> PgnTokenizer<'pgn> {
    pub fn new(source: &'pgn [u8]) -> Self {
        Self {
            len: source.len(),
            bytes: ByteLocations::new(source.iter().cloned().enumerate()).peekable(),
            errored: false,
            line: 0,
            col: 0,
        }
    }

    fn last_location(&self) -> Location {
        Location {
            line: self.line,
            col: self.col,
            byte_index: self.len,
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
    ) -> Span {
        let mut start = None;
        while let Some((location, byte)) = self.match_byte_if(func) {
            if start.is_none() {
                start = Some(location);
            }

            vec_to_append_to.push(byte);
        }

        Span {
            start: start.unwrap_or(self.last_location()),
            end: self.peek_location(),
        }
    }

    fn match_byte(&mut self, byte: u8) -> Option<(Location, u8)> {
        self.match_byte_if(&mut |other_byte| byte == other_byte)
    }

    fn match_byte_if<F: FnMut(u8) -> bool>(&mut self, func: &mut F) -> Option<(Location, u8)> {
        match self.bytes.peek() {
            None => return None,
            Some((loc, byte)) => {
                self.line = loc.line;
                self.col = loc.col;

                if func(*byte) {
                    return Some(
                        self.bytes
                            .next()
                            .expect("Should always be Some() since peek() returned Some()"),
                    );
                } else {
                    return None;
                }
            }
        }
    }

    fn peek_location(&mut self) -> Location {
        self.bytes
            .peek()
            .map(|(loc, _)| *loc)
            .unwrap_or(self.last_location())
    }
}

impl<'pgn> Iterator for PgnTokenizer<'pgn> {
    type Item = Result<PgnToken, PgnByteErr>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            return None;
        }

        if self.match_whitespace() {
            if let Some((start, b'%')) = self.bytes.peek() {
                let start = *start;
                self.bytes.next();
                let mut result = Vec::new();
                let span = self.match_byte_while(&mut |byte| byte != b'\n', &mut result);
                return Some(Ok(PgnToken {
                    span: Span {
                        start: start,
                        end: span.end,
                    },
                    kind: PgnTokenKind::EscapedLine(result),
                }));
            }
        }

        if let Some((start, byte)) = self.bytes.next() {
            match byte {
                // Do the self-closing tokens first since they have simpler handling.
                b'{' => {
                    let mut comment = Vec::new();
                    comment.push(b'{');

                    self.match_byte_while(&mut |byte| byte != b'}', &mut comment);

                    let found = if let Some((_, ch)) = self.bytes.next() {
                        if ch == b'}' {
                            comment.push(b'}');
                            return Some(Ok(PgnToken {
                                span: Span {
                                    start,
                                    end: self.peek_location(),
                                },
                                kind: PgnTokenKind::Comment(comment),
                            }));
                        } else {
                            Some(ch)
                        }
                    } else {
                        None
                    };

                    return Some(Err(PgnByteErr {
                        expected: vec!['}'],
                        not_expected: Vec::new(),
                        found,
                        location: self.peek_location(),
                    }));
                }
                b'[' => Some(Ok(PgnToken {
                    span: Span {
                        start,
                        end: self.peek_location(),
                    },
                    kind: PgnTokenKind::LeftSquareBracket,
                })),
                b']' => Some(Ok(PgnToken {
                    span: Span {
                        start,
                        end: self.peek_location(),
                    },
                    kind: PgnTokenKind::RightSquareBracket,
                })),
                b'<' => Some(Ok(PgnToken {
                    span: Span {
                        start,
                        end: self.peek_location(),
                    },
                    kind: PgnTokenKind::LeftAngleBracket,
                })),
                b'>' => Some(Ok(PgnToken {
                    span: Span {
                        start,
                        end: self.peek_location(),
                    },
                    kind: PgnTokenKind::RightAngleBracket,
                })),
                b'(' => Some(Ok(PgnToken {
                    span: Span {
                        start,
                        end: self.peek_location(),
                    },
                    kind: PgnTokenKind::LeftParen,
                })),
                b')' => Some(Ok(PgnToken {
                    span: Span {
                        start,
                        end: self.peek_location(),
                    },
                    kind: PgnTokenKind::RightParen,
                })),
                b'.' => Some(Ok(PgnToken {
                    span: Span {
                        start,
                        end: self.peek_location(),
                    },
                    kind: PgnTokenKind::Period,
                })),
                b'*' => Some(Ok(PgnToken {
                    span: Span {
                        start,
                        end: self.peek_location(),
                    },
                    kind: PgnTokenKind::Symbol("*".to_string()),
                })),
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
                    return Some(Ok(PgnToken {
                        span: Span {
                            start,
                            end: self.peek_location(),
                        },
                        kind: PgnTokenKind::Comment(comment),
                    }));
                }
                b'$' => {
                    let mut result = Vec::new();
                    if let Some(byte) = self.match_byte_if(&mut Self::is_decimal_digit) {
                        result.push(byte.1);
                    } else {
                        return Some(Err(PgnByteErr {
                            expected: vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
                            not_expected: Vec::new(),
                            found: match self.bytes.peek() {
                                None => None,
                                Some((_, ch)) => Some(*ch),
                            },
                            location: self.peek_location(),
                        }));
                    }

                    self.match_byte_while(&mut Self::is_decimal_digit, &mut result)
                        .end;

                    let string = unsafe { String::from_utf8_unchecked(result) };
                    let parsed = string.parse::<usize>().ok();

                    return Some(Ok(PgnToken {
                        span: Span {
                            start,
                            end: self.peek_location(),
                        },
                        kind: PgnTokenKind::NAG(Integer {
                            raw: string,
                            parsed,
                        }),
                    }));
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
                    if self.match_byte(b'"').is_some() {
                        return Some(Ok(PgnToken {
                            kind: PgnTokenKind::String(string_content),
                            span: Span {
                                start,
                                end: self.peek_location(),
                            },
                        }));
                    } else {
                        return Some(Err(match self.bytes.next() {
                            None => PgnByteErr {
                                expected: vec!['"'],
                                not_expected: Vec::new(),
                                found: None,
                                location: self.peek_location(),
                            },
                            Some((_, byte)) => PgnByteErr {
                                expected: vec!['"'],
                                not_expected: Vec::new(),
                                found: Some(byte),
                                location: self.peek_location(),
                            },
                        }));
                    }
                }
                b'0'..=b'9' => {
                    let mut result = Vec::new();
                    result.push(byte);
                    let matched_solidus = if byte == b'1' && self.match_byte(b'/').is_some() {
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
                            && self.match_byte(b'/').is_some()
                        {
                            // allow 1/2-1/2 to be counted as a symbol
                            result.push(b'/');
                            if self.match_byte(b'2').is_some() {
                                result.push(b'2');
                            }
                        } else {
                            for _ in 0..2 {
                                self.match_byte_if(&mut |byte| byte == b'!' || byte == b'?');
                            }
                        }

                        return Some(Ok(PgnToken {
                            span: Span {
                                start,
                                end: self.peek_location(),
                            },
                            kind: PgnTokenKind::Symbol(unsafe {
                                String::from_utf8_unchecked(result)
                            }),
                        }));
                    }

                    let string = unsafe { String::from_utf8_unchecked(result) };

                    let parsed = string.parse::<usize>().ok();
                    Some(Ok(PgnToken {
                        span: Span {
                            start: start,
                            end: self.peek_location(),
                        },
                        kind: PgnTokenKind::Integer(Integer {
                            raw: string,
                            parsed,
                        }),
                    }))
                }
                b'a'..=b'z' | b'A'..=b'Z' => {
                    let mut result = Vec::with_capacity(5);
                    result.push(byte);
                    self.match_byte_while(&mut Self::is_symbol_continuation, &mut result);
                    for _ in 0..2 {
                        self.match_byte_if(&mut |byte| byte == b'!' || byte == b'?');
                    }

                    return Some(Ok(PgnToken {
                        kind: PgnTokenKind::Symbol(unsafe { String::from_utf8_unchecked(result) }),
                        span: Span {
                            start,
                            end: self.peek_location(),
                        },
                    }));
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
                        found: Some(byte),
                        location: self.peek_location(),
                    }));
                }
            }
        } else {
            return None;
        }
    }
}

#[derive(Clone, Debug)]
enum PgnTokenKind {
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
    /// Numeric Annotation Glyph
    NAG(Integer),
    String(Vec<u8>),
    Integer(Integer),
    Symbol(String),
    Comment(Vec<u8>),
    EscapedLine(Vec<u8>),
}

#[derive(Debug)]
pub struct PgnToken {
    span: Span,
    kind: PgnTokenKind,
}

impl PgnToken {
    pub fn range(&self) -> Range<usize> {
        (&self.span).into()
    }

    pub fn kind_str(&self) -> &'static str {
        match self.kind {
            PgnTokenKind::LeftSquareBracket => "LeftSquareBracket",
            PgnTokenKind::RightSquareBracket => "RightSquareBracket",
            PgnTokenKind::LeftAngleBracket => "LeftAngleBracket",
            PgnTokenKind::RightAngleBracket => "RightAngleBracket",
            PgnTokenKind::LeftParen => "LeftParen",
            PgnTokenKind::RightParen => "RightParen",
            PgnTokenKind::Period => "Period",
            PgnTokenKind::NAG(_) => "NAG",
            PgnTokenKind::String(_) => "String",
            PgnTokenKind::Integer(_) => "Integer",
            PgnTokenKind::Symbol(_) => "Symbol",
            PgnTokenKind::Comment(_) => "Comment",
            PgnTokenKind::EscapedLine(_) => "EscapedLine",
        }
    }
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

impl PgnToken {
    fn kind(&self) -> &PgnTokenKind {
        &self.kind
    }
}

#[derive(Clone, Debug)]
struct Integer {
    #[allow(unused)]
    raw: String,
    #[allow(unused)]
    parsed: Option<usize>,
}

struct ByteLocations<ByteIndices>
where
    ByteIndices: Iterator<Item = (usize, u8)>,
{
    done: bool,
    source: Peekable<ByteIndices>,
    line: usize,
    col: usize,
    previous_was_new_line: bool,
}

impl<ByteIndices> ByteLocations<ByteIndices>
where
    ByteIndices: Iterator<Item = (usize, u8)>,
{
    pub fn new(source: ByteIndices) -> Self {
        Self {
            done: false,
            source: source.peekable(),
            line: 1,
            col: 0, // we will increment on every loop, so start at 0
            previous_was_new_line: false,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn col(&self) -> usize {
        if self.col == 0 {
            if self.line == 1 {
                1
            } else {
                0
            }
        } else {
            self.col
        }
    }

    fn attach_current_location(&self, byte_index: usize) -> Location {
        Location {
            byte_index,
            line: self.line,
            col: self.col,
        }
    }
}

impl<ByteIndices> Iterator for ByteLocations<ByteIndices>
where
    ByteIndices: Iterator<Item = (usize, u8)>,
{
    type Item = (Location, u8);

    fn next(&mut self) -> Option<Self::Item> {
        // Safety check. Don't overrun the end of any buffers
        if self.done {
            return None;
        }

        let next = match self.source.next() {
            None => {
                self.done = true;
                return None;
            }
            Some(next) => next,
        };

        let location = if self.previous_was_new_line {
            self.line += 1;
            self.col = 1;
            self.attach_current_location(next.0)
        } else {
            self.col += 1;
            self.attach_current_location(next.0)
        };

        self.previous_was_new_line = next.1 == b'\n';
        Some((location, next.1))
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_pgn, pgn_parser::PgnTokenizer};

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
        println!("{:#?}", parsed);
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
            println!("{:#?}", err);
        }

        for game in PgnParser::parse_pgn(pgn).unwrap() {
            println!("{:#?}", game);
        }
    }
}
