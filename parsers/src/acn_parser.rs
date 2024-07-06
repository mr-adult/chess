use std::fmt::Debug;

use chess_common::{File, Location, PieceKind, Rank};

pub fn parse_algebraic_notation(move_: &str) -> Option<PieceMove> {
    ACNParser::parse(move_)
}

pub struct PieceMove {
    /// The type of check this move resulted in
    pub check_kind: Check,
    pub move_kind: PieceMoveKind,
}

impl Debug for PieceMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl ToString for PieceMove {
    fn to_string(&self) -> String {
        let mut result = self.move_kind.to_string();

        match self.check_kind {
            Check::None => {}
            Check::Check => {
                result.push('+');
            }
            Check::Mate => {
                result.push('#');
            }
        }

        result
    }
}

pub enum PieceMoveKind {
    CastleKingside,
    CastleQueenside,
    Normal(NormalMove),
}

impl Debug for PieceMoveKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl ToString for PieceMoveKind {
    fn to_string(&self) -> String {
        match &self {
            PieceMoveKind::CastleKingside => "O-O".to_string(),
            PieceMoveKind::CastleQueenside => "O-O-O".to_string(),
            PieceMoveKind::Normal(normal_move) => normal_move.to_string(),
        }
    }
}

pub struct NormalMove {
    /// The piece being moved
    pub piece_kind: PieceKind,
    /// The destination square of the move
    pub destination: Location,
    /// The file from which the piece is moving (only given if necessary for disambiguation)
    pub disambiguation_file: Option<File>,
    /// The rank from which the piece is moving (only given if necessary for disambiguation)
    pub disambiguation_rank: Option<Rank>,
    pub is_capture: bool,
    /// None = no promotion
    pub promotion_kind: Option<PieceKind>,
    /// '?' and '!' annotations
    #[allow(unused)]
    pub move_suffix_annotations: [Option<SuffixAnnotation>; 2],
}

impl Debug for NormalMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl ToString for NormalMove {
    fn to_string(&self) -> String {
        let mut result = String::new();
        result.push(self.piece_kind.as_char());
        if result.as_bytes()[0] == b'P' {
            result.pop();
        }

        if let Some(disambiguation_file) = self.disambiguation_file {
            result.push(disambiguation_file.as_char());
        }

        if let Some(disambiguation_rank) = self.disambiguation_rank {
            result.push(disambiguation_rank.as_char());
        }

        if self.is_capture {
            result.push('x');
        }

        result.push(self.destination.file().as_char());
        result.push(self.destination.rank().as_char());

        if let Some(promotion_piece) = self.promotion_kind {
            result.push('=');
            result.push(promotion_piece.as_char());
        }

        result
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SuffixAnnotation {
    Exclamation,
    Question,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Check {
    None,
    Check,
    Mate,
}

pub(crate) struct ACNParser;
impl ACNParser {
    /// This method is currently unsafe outside of the pgn parser
    /// because it doesn't check the integrity of the string
    /// contents.
    fn parse(source: &str) -> Option<PieceMove> {
        if source.starts_with("O-O-O") {
            match source.len() {
                5 => {
                    return Some(PieceMove {
                        check_kind: Check::None,
                        move_kind: PieceMoveKind::CastleQueenside,
                    })
                }
                6 => {
                    let last_char = source.as_bytes()[3];
                    let check_kind = match last_char {
                        b'+' => Check::Check,
                        b'#' => Check::Mate,
                        _ => return None,
                    };

                    return Some(PieceMove {
                        check_kind,
                        move_kind: PieceMoveKind::CastleQueenside,
                    });
                }
                _ => return None,
            }
        }

        if source.starts_with("O-O") {
            match source.len() {
                3 => {
                    return Some(PieceMove {
                        check_kind: Check::None,
                        move_kind: PieceMoveKind::CastleKingside,
                    })
                }
                4 => {
                    let last_char = source.as_bytes()[3];
                    let check_kind = match last_char {
                        b'+' => Check::Check,
                        b'#' => Check::Mate,
                        _ => return None,
                    };

                    return Some(PieceMove {
                        check_kind,
                        move_kind: PieceMoveKind::CastleKingside,
                    });
                }
                _ => return None,
            }
        }

        let mut chars = source.chars().peekable();
        let piece_kind = match chars.peek()? {
            'P' => {
                // consume it!
                chars.next()?;
                PieceKind::Pawn
            }
            'N' => {
                // consume it!
                chars.next()?;
                PieceKind::Knight
            }
            'B' => {
                // consume it!
                chars.next()?;
                PieceKind::Bishop
            }
            'R' => {
                // consume it!
                chars.next()?;
                PieceKind::Rook
            }
            'Q' => {
                // consume it!
                chars.next()?;
                PieceKind::Queen
            }
            'K' => {
                // consume it!
                chars.next()?;
                PieceKind::King
            }
            _ => PieceKind::Pawn,
        };

        let mut current_char = chars.next()?;

        let mut files = Vec::<File>::with_capacity(2);
        if let Ok(file) = current_char.try_into() {
            files.push(file);
            current_char = chars.next()?;
        }

        let mut ranks = Vec::<Rank>::with_capacity(2);
        if let Ok(rank) = current_char.try_into() {
            ranks.push(rank);
            if let Some(ch) = chars.next() {
                current_char = ch;
            } else {
                return Some(PieceMove {
                    check_kind: Check::None,
                    move_kind: PieceMoveKind::Normal(NormalMove {
                        piece_kind,
                        destination: Location::new(files.pop()?, ranks.pop()?),
                        disambiguation_file: None, // if we ran out of characters this soon, there is not disambiguation.
                        disambiguation_rank: None, // if we ran out of characters this soon, there is not disambiguation.
                        is_capture: false,
                        promotion_kind: None,
                        move_suffix_annotations: [None, None],
                    }),
                });
            }
        }

        let is_capture = current_char == 'x';
        if is_capture {
            current_char = chars.next()?;
        }

        if let Ok(file) = current_char.try_into() {
            files.push(file);
            current_char = chars.next()?;
        }

        let mut check_kind = Check::None;
        let mut promotion: Option<PieceKind> = None;
        if let Ok(rank) = current_char.try_into() {
            ranks.push(rank);
            if let Some(_) = chars.peek() {
                if let Some(ch) = chars.next() {
                    current_char = ch;
                } else {
                    unreachable!();
                }
            }
        }

        if current_char == '=' {
            promotion = Some(chars.next()?.try_into().ok()?);
            if let Some(_) = chars.peek() {
                if let Some(ch) = chars.next() {
                    current_char = ch;
                } else {
                    unreachable!();
                }
            }
        }

        match current_char {
            '+' => {
                check_kind = Check::Check;
                if let Some(_) = chars.peek() {
                    if let Some(ch) = chars.next() {
                        current_char = ch;
                    } else {
                        unreachable!();
                    }
                }
            }
            '#' => {
                check_kind = Check::Mate;
                if let Some(_) = chars.peek() {
                    if let Some(ch) = chars.next() {
                        current_char = ch;
                    } else {
                        unreachable!();
                    }
                }
            }
            _ => {}
        }

        let mut annotations = [None, None];
        for i in 0..2 {
            if current_char == '?' || current_char == '!' {
                annotations[i] = Some(match current_char {
                    '?' => SuffixAnnotation::Question,
                    '!' => SuffixAnnotation::Exclamation,
                    _ => unreachable!(),
                });

                if let Some(_) = chars.peek() {
                    if let Some(ch) = chars.next() {
                        current_char = ch;
                    } else {
                        unreachable!();
                    }
                }
            } else {
                break;
            }
        }

        return Some(PieceMove {
            check_kind,
            move_kind: PieceMoveKind::Normal(NormalMove {
                piece_kind,
                destination: Location::new(files.pop()?, ranks.pop()?),
                disambiguation_file: files.pop(),
                disambiguation_rank: ranks.pop(),
                is_capture,
                promotion_kind: promotion,
                move_suffix_annotations: annotations,
            }),
        });
    }
}

#[cfg(test)]
mod tests {
    use chess_common::{File, Location, PieceKind, Rank};

    use crate::acn_parser::PieceMoveKind;

    use super::parse_algebraic_notation;
    use super::Check;

    #[test]
    fn parses_pawn_cases() {
        let move_ = parse_algebraic_notation("f4").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Pawn);
            assert!(!move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::f, Rank::Four));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }

        let move_ = parse_algebraic_notation("c4").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Pawn);
            assert!(!move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::c, Rank::Four));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }

        let move_ = parse_algebraic_notation("cxd5").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Pawn);
            assert!(move_details.is_capture);
            assert!(move_details.disambiguation_file == Some(File::c));
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::d, Rank::Five));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }

        let move_ = parse_algebraic_notation("fxg1=Q+").unwrap();
        assert!(move_.check_kind == Check::Check);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Pawn);
            assert!(move_details.is_capture);
            assert!(move_details.disambiguation_file == Some(File::f));
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::g, Rank::One));
            assert!(move_details.promotion_kind == Some(PieceKind::Queen));
        } else {
            panic!("Expected PieceMove::Normal");
        }
    }

    #[test]
    fn parses_knight_cases() {
        let move_ = parse_algebraic_notation("Nf3").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Knight);
            assert!(!move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::f, Rank::Three));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }

        let move_ = parse_algebraic_notation("Nxe5").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Knight);
            assert!(move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::e, Rank::Five));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }
    }

    #[test]
    fn parses_bishop_cases() {
        let move_ = parse_algebraic_notation("Bc4").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Bishop);
            assert!(!move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::c, Rank::Four));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }

        let move_ = parse_algebraic_notation("Bd2").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Bishop);
            assert!(!move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::d, Rank::Two));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }

        let move_ = parse_algebraic_notation("Bxb7").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Bishop);
            assert!(move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::b, Rank::Seven));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }
    }

    #[test]
    fn parses_rook_moves() {
        let move_ = parse_algebraic_notation("Re3").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Rook);
            assert!(!move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::e, Rank::Three));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }

        let move_ = parse_algebraic_notation("Rxc5").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Rook);
            assert!(move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::c, Rank::Five));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }

        let move_ = parse_algebraic_notation("Re5+").unwrap();
        assert!(move_.check_kind == Check::Check);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Rook);
            assert!(!move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::e, Rank::Five));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }
    }

    #[test]
    fn parses_queen_moves() {
        let move_ = parse_algebraic_notation("Qc5").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Queen);
            assert!(!move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::c, Rank::Five));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }

        let move_ = parse_algebraic_notation("Qa6xb7#").unwrap();
        assert!(move_.check_kind == Check::Mate);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Queen);
            assert!(move_details.is_capture);
            assert!(move_details.disambiguation_file == Some(File::a));
            assert!(move_details.disambiguation_rank == Some(Rank::Six));
            assert!(move_details.destination == Location::new(File::b, Rank::Seven));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }
    }

    #[test]
    fn parses_king_moves() {
        let move_ = parse_algebraic_notation("Kh3").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::King);
            assert!(!move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::h, Rank::Three));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }

        let move_ = parse_algebraic_notation("Kxa1#").unwrap();
        assert!(move_.check_kind == Check::Mate);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::King);
            assert!(move_details.is_capture);
            assert!(move_details.disambiguation_file.is_none());
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::a, Rank::One));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }
    }

    #[test]
    fn parses_pawn_capture_correctly() {
        let move_ = parse_algebraic_notation("axb4").unwrap();
        assert!(move_.check_kind == Check::None);
        if let PieceMoveKind::Normal(move_details) = move_.move_kind {
            assert!(move_details.piece_kind == PieceKind::Pawn);
            assert!(move_details.is_capture);
            assert!(move_details.disambiguation_file == Some(File::a));
            assert!(move_details.disambiguation_rank.is_none());
            assert!(move_details.destination == Location::new(File::b, Rank::Four));
            assert!(move_details.promotion_kind == None);
        } else {
            panic!("Expected PieceMove::Normal");
        }
    }
}
