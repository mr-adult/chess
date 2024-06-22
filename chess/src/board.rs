use std::{
    array::from_fn,
    iter,
    ops::{Index, IndexMut},
    str::FromStr,
};

use chess_common::{black, white, File, Location, Piece, PieceKind, Player, Rank};
use chess_parsers::{parse_fen, BoardLayout, FenErr};

use crate::{bitboard::BitBoard, legal_moves::LegalMovesIterator, Move};

#[derive(Debug)]
pub struct ErgonomicBoard {
    pieces: [[Option<Piece>; 8]; 8],
}

impl Index<Location> for ErgonomicBoard {
    type Output = Option<Piece>;

    fn index(&self, index: Location) -> &Self::Output {
        &self.pieces[index.rank().as_index()][index.file().as_index()]
    }
}

impl IndexMut<Location> for ErgonomicBoard {
    fn index_mut(&mut self, index: Location) -> &mut Self::Output {
        &mut self.pieces[index.rank().as_index()][index.file().as_index()]
    }
}

#[derive(Debug)]
pub struct Board {
    pub(crate) starting_position: BoardLayout,
    pub(crate) pawns: [BitBoard; 2],
    pub(crate) knights: [BitBoard; 2],
    pub(crate) bishops: [BitBoard; 2],
    pub(crate) rooks: [BitBoard; 2],
    pub(crate) queens: [BitBoard; 2],
    pub(crate) kings: [BitBoard; 2],
    /// A bitboard that represents occupied and unoccupied board squares
    pub(crate) mailbox: BitBoard,
    history: Vec<Move>,
    first_player_to_move: Player,
}

impl FromStr for Board {
    type Err = FenErr;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let layout = parse_fen(str)?;
        let player_to_move = layout.player_to_move();

        let mut result = Self {
            starting_position: layout,
            pawns: [BitBoard::default(), BitBoard::default()],
            knights: [BitBoard::default(), BitBoard::default()],
            bishops: [BitBoard::default(), BitBoard::default()],
            rooks: [BitBoard::default(), BitBoard::default()],
            queens: [BitBoard::default(), BitBoard::default()],
            kings: [BitBoard::default(), BitBoard::default()],
            mailbox: BitBoard::default(),
            history: Vec::new(),
            first_player_to_move: player_to_move,
        };

        for rank in Rank::all_ranks_ascending() {
            for file in File::all_files_ascending() {
                let location = Location::new(file, rank);
                let location_u64 = location.as_u64();

                match result.starting_position[location] {
                    None => continue,
                    Some(piece) => {
                        let piece_kind = piece.kind();
                        let player = piece.player();

                        match piece_kind {
                            PieceKind::Pawn => match player {
                                Player::White => {
                                    result.pawns[white!()].0 |= location_u64;
                                }
                                Player::Black => {
                                    result.pawns[black!()].0 |= location_u64;
                                }
                            },
                            PieceKind::Knight => match player {
                                Player::White => {
                                    result.knights[white!()].0 |= location_u64;
                                }
                                Player::Black => {
                                    result.knights[black!()].0 |= location_u64;
                                }
                            },
                            PieceKind::Bishop => match player {
                                Player::White => {
                                    result.bishops[white!()].0 |= location_u64;
                                }
                                Player::Black => {
                                    result.bishops[black!()].0 |= location_u64;
                                }
                            },
                            PieceKind::Rook => match player {
                                Player::White => {
                                    result.rooks[white!()].0 |= location_u64;
                                }
                                Player::Black => {
                                    result.rooks[black!()].0 |= location_u64;
                                }
                            },
                            PieceKind::Queen => match player {
                                Player::White => {
                                    result.queens[white!()].0 |= location_u64;
                                }
                                Player::Black => {
                                    result.queens[black!()].0 |= location_u64;
                                }
                            },
                            PieceKind::King => match player {
                                Player::White => {
                                    result.kings[white!()].0 |= location_u64;
                                }
                                Player::Black => {
                                    result.kings[black!()].0 |= location_u64;
                                }
                            },
                        }
                    }
                }
            }
        }

        result.update_mailbox();
        Ok(result)
    }
}

impl ToString for Board {
    fn to_string(&self) -> String {
        self.assert_board_integrity();

        // FEN notation can only be 84 bytes max
        let mut result = String::with_capacity(84);
        for (rank_num, rank) in Rank::all_ranks_ascending().rev().enumerate() {
            if rank_num != 0 {
                result.push('/');
            }

            let mut empty_spaces = 0;
            for file in File::all_files_ascending() {
                match self.at(Location::new(file, rank)) {
                    None => empty_spaces += 1,
                    Some(piece) => {
                        if empty_spaces > 0 {
                            result.push_str(&empty_spaces.to_string());
                            empty_spaces = 0;
                        }

                        result.push(piece.to_fen());
                    }
                }
            }
            if empty_spaces > 0 {
                result.push_str(&empty_spaces.to_string());
            }
        }

        result.push(' ');
        result.push(self.get_player_to_move().as_char());

        result.push(' ');
        let mut any_castling_allowed = false;
        if self.white_can_castle_kingside() {
            any_castling_allowed = true;
            result.push('K');
        }
        if self.white_can_castle_queenside() {
            any_castling_allowed = true;
            result.push('Q');
        }
        if self.black_can_castle_kingside() {
            any_castling_allowed = true;
            result.push('k');
        }
        if self.black_can_castle_queenside() {
            any_castling_allowed = true;
            result.push('q');
        }
        if !any_castling_allowed {
            result.push('-')
        }

        result.push(' ');
        if let Some(location) = self.en_passant_target_square() {
            result.push(location.file().as_char());
            result.push(location.rank().as_char());
        } else {
            result.push('-');
        }

        result.push(' ');
        result.push_str(&self.half_moves_played().to_string());

        result.push(' ');
        result.push_str(&self.full_moves_played().to_string());

        result
    }
}

impl Board {
    fn update_mailbox(&mut self) {
        // first, clear it
        self.mailbox.0 &= 0_u64;
        for bitboard in [
            &self.pawns,
            &self.knights,
            &self.bishops,
            &self.rooks,
            &self.queens,
            &self.kings,
        ]
        .into_iter()
        .flat_map(|arr| arr)
        {
            self.mailbox.0 |= bitboard.0
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn assert_board_integrity(&self) {
        for (i, bitboard_1) in self.all_bitboards().enumerate() {
            for (j, bitboard_2) in self.all_bitboards().enumerate() {
                if bitboard_1 as *const BitBoard == bitboard_2 as *const BitBoard {
                    continue;
                }

                if bitboard_1 as *const BitBoard == &self.mailbox as *const BitBoard {
                    continue;
                }

                if bitboard_2 as *const BitBoard == &self.mailbox as *const BitBoard {
                    continue;
                }

                if bitboard_1.0 & bitboard_2.0 != 0 {
                    let locations = Location::from_bitboard(bitboard_1.0 & bitboard_2.0)
                        .map(|loc| format!("{:?}", loc))
                        .collect::<Vec<_>>()
                        .join(", ");
                    panic!(
                        "Found conflicting bitboards at indexes {i} and {j}. Locations of contention: {}. Board: {:?}",
                        locations,
                        self
                    );
                }
            }
        }
    }

    fn all_bitboards(&self) -> impl Iterator<Item = &BitBoard> {
        return self
            .pawns
            .iter()
            .chain(self.knights.iter())
            .chain(self.bishops.iter())
            .chain(self.rooks.iter())
            .chain(self.queens.iter())
            .chain(self.kings.iter())
            .chain(iter::once(&self.mailbox));
    }

    pub(crate) fn create_mailbox_for_player(&self, player: Player) -> BitBoard {
        let player_index = player.as_index();
        return BitBoard::new(
            self.pawns[player_index].0
                | self.knights[player_index].0
                | self.bishops[player_index].0
                | self.rooks[player_index].0
                | self.queens[player_index].0
                | self.kings[player_index].0,
        );
    }

    pub(crate) fn get_player_to_move(&self) -> Player {
        if self.history.len() % 2 == 0 {
            return self.first_player_to_move;
        } else {
            return self.first_player_to_move.other_player();
        }
    }

    pub(crate) fn player_can_castle_kingside(&self, player: &Player) -> bool {
        match player {
            Player::Black => self.black_can_castle_kingside(),
            Player::White => self.white_can_castle_kingside(),
        }
    }

    fn white_can_castle_kingside(&self) -> bool {
        if !self.starting_position.white_can_castle_kingside() {
            return false;
        }

        return !self.history.iter().any(|move_| {
            move_.from == Location::new(File::e, Rank::One)
                || move_.from == Location::new(File::h, Rank::One)
        });
    }

    fn black_can_castle_kingside(&self) -> bool {
        if !self.starting_position.black_can_castle_kingside() {
            return false;
        }

        return !self.history.iter().any(|move_| {
            move_.from == Location::new(File::e, Rank::Eight)
                || move_.from == Location::new(File::h, Rank::Eight)
        });
    }

    pub(crate) fn player_can_castle_queenside(&self, player: &Player) -> bool {
        match player {
            Player::Black => self.black_can_castle_queenside(),
            Player::White => self.white_can_castle_queenside(),
        }
    }

    fn white_can_castle_queenside(&self) -> bool {
        if !self.starting_position.white_can_castle_queenside() {
            return false;
        }

        return !self.history.iter().any(|move_| {
            move_.from == Location::new(File::e, Rank::One)
                || move_.from == Location::new(File::a, Rank::One)
        });
    }

    fn black_can_castle_queenside(&self) -> bool {
        if !self.starting_position.black_can_castle_queenside() {
            return false;
        }

        return !self.history.iter().any(|move_| {
            move_.from == Location::new(File::e, Rank::Eight)
                || move_.from == Location::new(File::a, Rank::Eight)
        });
    }

    fn half_moves_played(&self) -> u8 {
        self.starting_position.half_move_counter() + self.history.len() as u8
    }

    fn full_moves_played(&self) -> u8 {
        let history_len_u8 = self.history.len() as u8;
        match self.starting_position.player_to_move() {
            Player::White => self.starting_position.full_move_counter() + history_len_u8 / 2,
            Player::Black => {
                self.starting_position.full_move_counter() + history_len_u8 / 2 + history_len_u8 % 2
            }
        }
    }

    pub(crate) fn en_passant_target_square(&self) -> Option<Location> {
        if let Some(last_move) = self.history.last() {
            if let Some(last_moved) = self.at(last_move.to) {
                if let PieceKind::Pawn = last_moved.kind() {
                    // It may not be necessary to check the files, but it is safer.
                    if last_move.to.file() == last_move.from.file()
                        && (last_move.to.rank().as_int() - last_move.from.rank().as_int()).abs()
                            == 2
                    {
                        let file = last_move.to.file();
                        let rank = match last_moved.player() {
                            Player::White => Rank::Three,
                            Player::Black => Rank::Six,
                        };
                        let rank = Rank::try_from(rank)
                            .expect("Integrity issue: as_int() should always be re-parsable");
                        return Some(Location::new(file, rank));
                    }
                }
            }
        } else if let Some(location) = self.starting_position.en_passant_target_square() {
            return Some(location);
        }

        return None;
    }

    fn at(&self, location: Location) -> Option<Piece> {
        let location_bits = location.as_u64();

        if location_bits == 0 {
            return None;
        }
        if location_bits & self.mailbox.0 == 0 {
            return None;
        }

        // pawns are most common, so check them first
        if self.pawns[white!()].0 & location_bits != 0 {
            return Some(Piece::new(Player::White, PieceKind::Pawn));
        }

        if self.pawns[black!()].0 & location_bits != 0 {
            return Some(Piece::new(Player::Black, PieceKind::Pawn));
        }

        // Rooks are equally as common as bishops and knights, but tend
        // to be more prevalent in the mid to end game, so check them
        // before bishops/knights
        if self.rooks[white!()].0 & location_bits != 0 {
            return Some(Piece::new(Player::White, PieceKind::Rook));
        }

        if self.rooks[black!()].0 & location_bits != 0 {
            return Some(Piece::new(Player::Black, PieceKind::Rook));
        }

        if self.knights[white!()].0 & location_bits != 0 {
            return Some(Piece::new(Player::White, PieceKind::Knight));
        }

        if self.knights[black!()].0 & location_bits != 0 {
            return Some(Piece::new(Player::Black, PieceKind::Knight));
        }

        if self.bishops[white!()].0 & location_bits != 0 {
            return Some(Piece::new(Player::White, PieceKind::Bishop));
        }

        if self.bishops[black!()].0 & location_bits != 0 {
            return Some(Piece::new(Player::Black, PieceKind::Bishop));
        }

        // Only 1 queen/king per side, so check them last
        if self.queens[white!()].0 & location_bits != 0 {
            return Some(Piece::new(Player::White, PieceKind::Queen));
        }

        if self.queens[black!()].0 & location_bits != 0 {
            return Some(Piece::new(Player::Black, PieceKind::Queen));
        }

        if self.kings[white!()].0 & location_bits != 0 {
            return Some(Piece::new(Player::White, PieceKind::King));
        }

        if self.kings[black!()].0 & location_bits != 0 {
            return Some(Piece::new(Player::Black, PieceKind::King));
        }

        return None;
    }

    pub fn into_ergo_board(&self) -> ErgonomicBoard {
        self.assert_board_integrity();
        let mut result = ErgonomicBoard {
            pieces: from_fn(|_| from_fn(|_| None)),
        };

        for rank in Rank::all_ranks_ascending() {
            for file in File::all_files_ascending() {
                let location = Location::new(file, rank);
                result[location] = self.at(location);
            }
        }

        result
    }

    pub fn legal_moves<'a>(&'a self) -> impl Iterator<Item = Move> + 'a {
        LegalMovesIterator::for_board(self)
    }

    pub fn make_move(&mut self, move_: Move) -> Result<(), ()> {
        if !self.legal_moves().any(|legal_move| legal_move == move_) {
            return Err(());
        }

        match self.at(move_.from) {
            None => Err(()),
            Some(piece_to_move) => {
                let player_to_move = piece_to_move.player();
                let to = move_.to.as_u64();

                let en_passant_target = self.en_passant_target_square();
                let capture_to =
                    if en_passant_target.is_some() && en_passant_target.unwrap() == move_.to {
                        let en_passant_target = en_passant_target.unwrap();
                        let real_pawn_location;
                        match player_to_move {
                            Player::Black => {
                                real_pawn_location = BitBoard::new(en_passant_target.as_u64()).up();
                            }
                            Player::White => {
                                real_pawn_location = BitBoard::new(en_passant_target.as_u64()).down();
                            }
                        }
                        real_pawn_location.0
                    } else {
                        to
                    };

                if let Some(captured_piece) = self
                    .at(Location::try_from(capture_to)
                        .expect(Location::failed_from_usize_message()))
                {
                    let opponent = captured_piece.player().as_index();
                    match captured_piece.kind() {
                        PieceKind::Pawn => {
                            self.pawns[opponent].0 ^= capture_to;
                        }
                        PieceKind::Knight => {
                            self.knights[opponent].0 ^= capture_to;
                        }
                        PieceKind::Bishop => {
                            self.bishops[opponent].0 ^= capture_to;
                        }
                        PieceKind::Rook => {
                            self.rooks[opponent].0 ^= capture_to;
                        }
                        PieceKind::Queen => {
                            self.queens[opponent].0 ^= capture_to;
                        }
                        PieceKind::King => {
                            self.kings[opponent].0 ^= capture_to;
                        }
                    }
                }

                let player = player_to_move.as_index();
                let from = move_.from.as_u64();

                match piece_to_move.kind() {
                    PieceKind::Pawn => {
                        self.pawns[player].0 ^= from;
                        self.pawns[player].0 ^= to;
                    }
                    PieceKind::Knight => {
                        self.knights[player].0 ^= from;
                        self.knights[player].0 ^= to;
                    }
                    PieceKind::Bishop => {
                        self.bishops[player].0 ^= from;
                        self.bishops[player].0 ^= to;
                    }
                    PieceKind::Rook => {
                        self.rooks[player].0 ^= from;
                        self.rooks[player].0 ^= to;
                    }
                    PieceKind::Queen => {
                        self.queens[player].0 ^= from;
                        self.queens[player].0 ^= to;
                    }
                    PieceKind::King => {
                        self.kings[player].0 ^= from;
                        self.kings[player].0 ^= to;
                    }
                }

                self.history.push(move_);
                self.update_mailbox();
                return Ok(());
            }
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}
