use std::fmt::Display;
use std::iter::Enumerate;
use std::ops::Index;
use std::str::Chars;
use std::{fmt::Debug, iter::Peekable};

use crate::PieceLocations;
use chess_common::{File, Location, Piece, PieceKind, Player, Rank};

#[derive(Debug)]
pub struct FenErr {
    failed_at_char_index: u8,
}

impl FenErr {
    fn new(char_index: u8) -> Self {
        Self {
            failed_at_char_index: char_index,
        }
    }
}

impl Display for FenErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = format!(
            "Failed to parse FEN. Error at character number {} (0-indexed)",
            self.failed_at_char_index
        );
        write!(f, "{}", str)
    }
}

pub(crate) struct FenParser<'fen> {
    chars: Peekable<Enumerate<Chars<'fen>>>,
    last_index: usize,
}

impl<'fen> FenParser<'fen> {
    pub(crate) fn parse_fen(input: &'fen str) -> Result<BoardLayout, FenErr> {
        let mut parser = Self {
            chars: input.chars().enumerate().peekable(),
            last_index: 0,
        };

        let piece_locations = parser.parse_piece_placement()?;

        parser.match_char_or_err(' ')?;

        let mut player_to_move = Player::White;
        parser.match_char_or_err_if(|ch| match ch {
            'b' => {
                player_to_move = Player::Black;
                true
            }
            'w' => {
                player_to_move = Player::White;
                true
            }
            _ => false,
        })?;

        parser.match_char_or_err(' ')?;

        let white_can_castle_kingside;
        let white_can_castle_queenside;
        let black_can_castle_kingside;
        let black_can_castle_queenside;
        if parser.match_char('-') {
            white_can_castle_kingside = false;
            white_can_castle_queenside = false;
            black_can_castle_kingside = false;
            black_can_castle_queenside = false;
        } else {
            white_can_castle_kingside = parser.match_char('K');
            white_can_castle_queenside = parser.match_char('Q');
            black_can_castle_kingside = parser.match_char('k');
            if white_can_castle_kingside || white_can_castle_queenside || black_can_castle_kingside
            {
                black_can_castle_queenside = parser.match_char('q');
            } else {
                let result = parser.match_char_or_err('q');
                black_can_castle_queenside = result.is_ok();
                let _ = result?;
            }
        }

        parser.match_char_or_err(' ')?;

        let en_passant = if parser.match_char('-') {
            None
        } else {
            let mut en_passant_file = File::a;
            parser.match_char_or_err_if(|ch| match File::try_from(ch) {
                Ok(file) => {
                    en_passant_file = file;
                    true
                }
                Err(_) => false,
            })?;

            let mut en_passant_rank = Rank::Three;
            parser.match_char_or_err_if(|ch| {
                match ch {
                    '3' => en_passant_rank = Rank::Three,
                    '6' => en_passant_rank = Rank::Six,
                    _ => return false,
                }
                return true;
            })?;

            Some(Location::new(en_passant_file, en_passant_rank))
        };

        parser.match_char_or_err(' ')?;

        let mut half_move_counter = 0;
        parser.match_char_or_err_if(|ch| match ch {
            '0'..='9' => {
                half_move_counter = (ch as u8) - b'0';
                true
            }
            _ => false,
        })?;

        parser.match_char_if(|ch| match ch {
            '0'..='9' => {
                half_move_counter *= 10;
                half_move_counter += (ch as u8) - b'0';
                true
            }
            _ => false,
        });

        parser.match_char_or_err(' ')?;

        let mut full_move_counter = 0;
        parser.match_char_if(|ch| match ch {
            '1'..='9' => {
                full_move_counter = ch as u8 - b'0';
                true
            }
            _ => false,
        });

        parser.match_char_if(|ch| match ch {
            '0'..='9' => {
                full_move_counter *= 10;
                full_move_counter += ch as u8 - b'0';
                true
            }
            _ => false,
        });

        Ok(BoardLayout {
            piece_locations,
            player_to_move,
            white_can_castle_kingside,
            white_can_castle_queenside,
            black_can_castle_kingside,
            black_can_castle_queenside,
            en_passant,
            half_move_counter,
            full_move_counter,
        })
    }

    fn parse_piece_placement(&mut self) -> Result<PieceLocations, FenErr> {
        let mut result = PieceLocations::default();
        let mut num_to_skip = 0;
        for rank in (0_u8..8_u8).rev() {
            // It's important that we reset this at every rank to
            // avoid allowing malformed FEN like ppppp8/rkb/...
            if num_to_skip != 0 {
                return Err(FenErr::new(self.last_index as u8));
            }
            num_to_skip = 0;

            for file in 0_u8..8_u8 {
                if num_to_skip > 0 {
                    num_to_skip -= 1;
                    continue;
                }

                if file == 0 && rank != 7 {
                    self.match_char_or_err('/')?;
                }

                let location = &Location::new(
                    File::try_from(file).expect("File to be a valid file"),
                    Rank::try_from(rank).expect("Rank to be a valid rank"),
                );

                let matched = self.match_char_if(|ch| {
                    match ch {
                        'p' => result[&location] = Some(Piece::new(Player::Black, PieceKind::Pawn)),
                        'P' => result[&location] = Some(Piece::new(Player::White, PieceKind::Pawn)),
                        'n' => {
                            result[&location] = Some(Piece::new(Player::Black, PieceKind::Knight))
                        }
                        'N' => {
                            result[&location] = Some(Piece::new(Player::White, PieceKind::Knight))
                        }
                        'b' => {
                            result[&location] = Some(Piece::new(Player::Black, PieceKind::Bishop))
                        }
                        'B' => {
                            result[&location] = Some(Piece::new(Player::White, PieceKind::Bishop))
                        }
                        'r' => result[&location] = Some(Piece::new(Player::Black, PieceKind::Rook)),
                        'R' => result[&location] = Some(Piece::new(Player::White, PieceKind::Rook)),
                        'q' => {
                            result[&location] = Some(Piece::new(Player::Black, PieceKind::Queen))
                        }
                        'Q' => {
                            result[&location] = Some(Piece::new(Player::White, PieceKind::Queen))
                        }
                        'k' => result[&location] = Some(Piece::new(Player::Black, PieceKind::King)),
                        'K' => result[&location] = Some(Piece::new(Player::White, PieceKind::King)),
                        '1'..='8' => {
                            num_to_skip = ch as u32 - 0x31_u32; // 0x31 is '1'
                        }
                        _ => {
                            return false;
                        }
                    }
                    return true;
                });

                if !matched {
                    return Err(FenErr::new(self.last_index as u8));
                }
            }
        }

        Ok(result)
    }

    fn match_char_or_err(&mut self, ch: char) -> Result<(), FenErr> {
        if self.match_char(ch) {
            Ok(())
        } else {
            Err(FenErr::new(self.last_index as u8))
        }
    }

    fn match_char(&mut self, ch: char) -> bool {
        self.match_char_if(|other| other == ch)
    }

    fn match_char_or_err_if<P: FnOnce(char) -> bool>(
        &mut self,
        predicate: P,
    ) -> Result<(), FenErr> {
        if self.match_char_if(predicate) {
            Ok(())
        } else {
            Err(FenErr::new(self.last_index as u8))
        }
    }

    fn match_char_if<P: FnOnce(char) -> bool>(&mut self, predicate: P) -> bool {
        match self.chars.peek() {
            None => false,
            Some((index, ch)) => {
                self.last_index = *index;
                if predicate(*ch) {
                    self.chars.next();
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct BoardLayout {
    piece_locations: PieceLocations,
    player_to_move: Player,
    white_can_castle_kingside: bool,
    white_can_castle_queenside: bool,
    black_can_castle_kingside: bool,
    black_can_castle_queenside: bool,
    en_passant: Option<Location>,
    half_move_counter: u8,
    full_move_counter: u8,
}

impl BoardLayout {
    pub const fn new(
        piece_locations: PieceLocations,
        player_to_move: Player,
        white_can_castle_kingside: bool,
        white_can_castle_queenside: bool,
        black_can_castle_kingside: bool,
        black_can_castle_queenside: bool,
        en_passant_target: Option<Location>,
        half_moves: u8,
        full_moves: u8,
    ) -> Self {
        Self {
            piece_locations,
            player_to_move,
            white_can_castle_kingside,
            white_can_castle_queenside,
            black_can_castle_kingside,
            black_can_castle_queenside,
            en_passant: en_passant_target,
            half_move_counter: half_moves,
            full_move_counter: full_moves,
        }
    }

    pub const fn player_to_move(&self) -> Player {
        self.player_to_move
    }

    pub fn white_can_castle_kingside(&self) -> bool {
        self.white_can_castle_kingside
    }

    pub fn white_can_castle_queenside(&self) -> bool {
        self.white_can_castle_queenside
    }

    pub fn black_can_castle_kingside(&self) -> bool {
        self.black_can_castle_kingside
    }

    pub fn black_can_castle_queenside(&self) -> bool {
        self.black_can_castle_queenside
    }

    pub fn en_passant_target_square(&self) -> Option<&Location> {
        self.en_passant.as_ref()
    }

    pub const fn half_move_counter(&self) -> u8 {
        self.half_move_counter
    }

    pub const fn full_move_counter(&self) -> u8 {
        self.full_move_counter
    }
}

impl ToString for BoardLayout {
    fn to_string(&self) -> String {
        let mut fen = Vec::with_capacity(84); // 84 is the maximum length of a FEN string.
        for (i, rank) in Rank::all_ranks_ascending().rev().enumerate() {
            let mut num_empties = 0;

            if i != 0 {
                fen.push(b'/');
            }

            for file in File::all_files_ascending() {
                match self[&Location::new(file, rank)] {
                    None => num_empties += 1,
                    Some(piece) => {
                        if num_empties > 0 {
                            fen.push(num_empties + b'0');
                            num_empties = 0;
                        }
                        match piece.player() {
                            Player::White => {
                                fen.push(piece.kind().as_char().to_ascii_uppercase() as u8)
                            }
                            Player::Black => {
                                fen.push(piece.kind().as_char().to_ascii_lowercase() as u8)
                            }
                        }
                    }
                }
            }

            if num_empties > 0 {
                fen.push(num_empties + b'0');
            }
        }

        fen.push(b' ');
        fen.push(self.player_to_move().as_char() as u8);
        fen.push(b' ');

        let mut any_valid_castles = false;
        if self.white_can_castle_kingside() {
            fen.push(b'K');
            any_valid_castles = true;
        }
        if self.white_can_castle_queenside() {
            fen.push(b'Q');
            any_valid_castles = true;
        }
        if self.black_can_castle_kingside() {
            fen.push(b'k');
            any_valid_castles = true;
        }
        if self.black_can_castle_queenside() {
            fen.push(b'q');
            any_valid_castles = true;
        }
        if !any_valid_castles {
            fen.push(b'-');
        }

        fen.push(b' ');
        match self.en_passant_target_square() {
            None => fen.push(b'-'),
            Some(location) => {
                fen.push(location.file().as_char() as u8);
                fen.push(location.rank().as_char() as u8);
            }
        }

        fen.push(b' ');
        for ch in self.half_move_counter().to_string().chars() {
            fen.push(ch as u8);
        }

        fen.push(b' ');
        for ch in self.full_move_counter().to_string().chars() {
            fen.push(ch as u8);
        }

        unsafe { String::from_utf8_unchecked(fen) }
    }
}

impl Index<&Location> for BoardLayout {
    type Output = Option<Piece>;

    fn index(&self, index: &Location) -> &Self::Output {
        &self.piece_locations[index]
    }
}

#[cfg(test)]
mod tests {
    use chess_common::{File, Location, Piece, PieceKind, Player, Rank};

    use crate::parse_fen;

    #[test]
    fn parses_default_board_state() {
        let input_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let layout = parse_fen(&input_fen).unwrap();

        // Rooks
        assert!(
            layout[&Location::new(File::a, Rank::One)]
                == Some(Piece::new(Player::White, PieceKind::Rook))
        );
        assert!(
            layout[&Location::new(File::h, Rank::One)]
                == Some(Piece::new(Player::White, PieceKind::Rook))
        );
        assert!(
            layout[&Location::new(File::a, Rank::Eight)]
                == Some(Piece::new(Player::Black, PieceKind::Rook))
        );
        assert!(
            layout[&Location::new(File::h, Rank::Eight)]
                == Some(Piece::new(Player::Black, PieceKind::Rook))
        );

        // Knights
        assert!(
            layout[&Location::new(File::b, Rank::One)]
                == Some(Piece::new(Player::White, PieceKind::Knight))
        );
        assert!(
            layout[&Location::new(File::g, Rank::One)]
                == Some(Piece::new(Player::White, PieceKind::Knight))
        );
        assert!(
            layout[&Location::new(File::b, Rank::Eight)]
                == Some(Piece::new(Player::Black, PieceKind::Knight))
        );
        assert!(
            layout[&Location::new(File::g, Rank::Eight)]
                == Some(Piece::new(Player::Black, PieceKind::Knight))
        );

        // Bishops
        assert!(
            layout[&Location::new(File::c, Rank::One)]
                == Some(Piece::new(Player::White, PieceKind::Bishop))
        );
        assert!(
            layout[&Location::new(File::f, Rank::One)]
                == Some(Piece::new(Player::White, PieceKind::Bishop))
        );
        assert!(
            layout[&Location::new(File::c, Rank::Eight)]
                == Some(Piece::new(Player::Black, PieceKind::Bishop))
        );
        assert!(
            layout[&Location::new(File::f, Rank::Eight)]
                == Some(Piece::new(Player::Black, PieceKind::Bishop))
        );

        // Queens
        assert!(
            layout[&Location::new(File::d, Rank::One)]
                == Some(Piece::new(Player::White, PieceKind::Queen))
        );
        assert!(
            layout[&Location::new(File::d, Rank::Eight)]
                == Some(Piece::new(Player::Black, PieceKind::Queen))
        );

        // Kings
        assert!(
            layout[&Location::new(File::e, Rank::One)]
                == Some(Piece::new(Player::White, PieceKind::King))
        );
        assert!(
            layout[&Location::new(File::e, Rank::Eight)]
                == Some(Piece::new(Player::Black, PieceKind::King))
        );

        // Pawns
        for file in File::all_files_ascending() {
            assert!(
                layout[&Location::new(file, Rank::Two)]
                    == Some(Piece::new(Player::White, PieceKind::Pawn))
            );
            assert!(
                layout[&Location::new(file, Rank::Seven)]
                    == Some(Piece::new(Player::Black, PieceKind::Pawn))
            );
        }

        // empties
        for file in File::all_files_ascending() {
            for rank in [Rank::Three, Rank::Four, Rank::Five, Rank::Six] {
                assert!(layout[&Location::new(file, rank)].is_none());
            }
        }

        assert!(layout.player_to_move() == Player::White);
        assert!(layout.white_can_castle_kingside());
        assert!(layout.white_can_castle_queenside());
        assert!(layout.black_can_castle_kingside());
        assert!(layout.black_can_castle_queenside());
        assert!(layout.en_passant_target_square().is_none());
        assert!(layout.half_move_counter() == 0);
        assert!(layout.full_move_counter() == 1);

        assert!(layout.to_string() == input_fen);
    }

    #[test]
    fn parses_valid_fen_strings() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 1 1";
        parse_fen(fen).unwrap();
    }
}
