use std::{
    array::from_fn,
    iter,
    ops::{Index, IndexMut},
    str::FromStr,
};

use chess_common::{black, white, File, Location, Piece, PieceKind, Player, Rank};
use chess_parsers::{parse_fen, BoardLayout, FenErr};

use crate::{
    bitboard::BitBoard, chess_move::SelectedMove, legal_moves::LegalMovesIterator,
    possible_moves::PossibleMovesIterator, Move,
};

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
    history: Vec<UndoableMove>,
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
                match self.at(&Location::new(file, rank)) {
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

        return !self.history.iter().any(|undoable_move| {
            let move_ = undoable_move.move_();
            move_.from == Location::new(File::e, Rank::One)
                || move_.from == Location::new(File::h, Rank::One)
        });
    }

    fn black_can_castle_kingside(&self) -> bool {
        if !self.starting_position.black_can_castle_kingside() {
            return false;
        }

        return !self.history.iter().any(|undoable_move| {
            let move_ = undoable_move.move_();
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

        return !self.history.iter().any(|undoable_move| {
            let move_ = undoable_move.move_();
            move_.from == Location::new(File::e, Rank::One)
                || move_.from == Location::new(File::a, Rank::One)
        });
    }

    fn black_can_castle_queenside(&self) -> bool {
        if !self.starting_position.black_can_castle_queenside() {
            return false;
        }

        return !self.history.iter().any(|undoable_move| {
            let move_ = undoable_move.move_();
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
            let last_move = last_move.move_();
            if let Some(last_moved) = self.at(&last_move.to) {
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

    fn at(&self, location: &Location) -> Option<Piece> {
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
                result[location] = self.at(&location);
            }
        }

        result
    }

    pub fn legal_moves<'board>(&'board self) -> LegalMovesIterator<'board> {
        LegalMovesIterator::for_board(self)
    }

    pub fn possible_moves<'board>(&'board self) -> PossibleMovesIterator<'board> {
        PossibleMovesIterator::new(self.legal_moves())
    }

    /// Makes the selected move.
    ///
    /// If the move is not valid, returns an Error with the reason it is invalid.
    pub fn make_move(&mut self, move_: SelectedMove) -> Result<(), MoveErr> {
        if !self
            .legal_moves()
            .any(|legal_move| *legal_move.move_() == *move_.move_())
        {
            return Err(MoveErr::IllegalMove);
        }

        self.make_move_unchecked(move_)
    }

    const WHITE_PAWN: Piece = Piece::new(Player::White, PieceKind::Pawn);
    const BLACK_PAWN: Piece = Piece::new(Player::Black, PieceKind::Pawn);

    /// Makes a move while skipping some of the more expensive validity checks.
    ///
    /// This function does not check if the move is legal.
    ///
    /// This function will still return an error if
    /// 1. the move's from location does not contain a piece
    /// 2. the piece being promoted is not a pawn
    /// 3. the promotion_kind is to a pawn to king
    /// as all of these checks are cheap.
    pub fn make_move_unchecked(
        &mut self,
        selected_move: SelectedMove,
    ) -> Result<(), MoveErr> {
        let promotion_kind = selected_move.promotion_kind();
        // Can't promote to king or pawn!
        if let Some(PieceKind::King | PieceKind::Pawn) = promotion_kind {
            return Err(MoveErr::IllegalPromotionPieceChoice);
        }

        let move_ = selected_move.move_();
        match self.at(&move_.from) {
            None => Err(MoveErr::NoPieceAtFromLocation),
            Some(piece_to_move) => {
                if promotion_kind.is_some() && piece_to_move.kind() != PieceKind::Pawn {
                    return Err(MoveErr::PromotionTargetNotPawn);
                }

                let to_rank = move_.to.rank();
                if piece_to_move.kind() == PieceKind::Pawn
                    && (to_rank == Rank::One || to_rank == Rank::Eight)
                    && promotion_kind.is_none()
                {
                    return Err(MoveErr::MislabeledPromotion);
                }

                let move_kind = self.classify_move(&piece_to_move, selected_move);

                match &move_kind {
                    UndoableMove::EnPassant {
                        move_,
                        captured_pawn_location,
                    } => unsafe {
                        self.remove_piece_at(
                            captured_pawn_location,
                            match piece_to_move.player().other_player() {
                                Player::White => &Self::WHITE_PAWN,
                                Player::Black => &Self::BLACK_PAWN,
                            },
                        );

                        self.move_piece(move_, &piece_to_move);
                    },
                    UndoableMove::Castles { move_, rook_move } => {
                        let rook = Piece::new(piece_to_move.player(), PieceKind::Rook);
                        unsafe {
                            self.move_piece(rook_move, &rook);
                            self.move_piece(move_, &piece_to_move);
                        }
                    }
                    UndoableMove::Normal { move_ } => unsafe {
                        self.move_piece(move_, &piece_to_move);
                    },
                    UndoableMove::Capture {
                        move_,
                        captured_piece,
                    } => unsafe {
                        self.remove_piece_at(&move_.to, captured_piece);
                        self.move_piece(move_, &piece_to_move);
                    },
                    UndoableMove::Promotion { move_, promoted_to } => unsafe {
                        self.remove_piece_at(&move_.from, &piece_to_move);
                        self.add_piece_at(
                            &move_.to,
                            &Piece::new(piece_to_move.player(), *promoted_to),
                        )
                    },
                    UndoableMove::CapturePromotion {
                        move_,
                        captured_piece,
                        promoted_to,
                    } => unsafe {
                        self.remove_piece_at(&move_.to, captured_piece);
                        self.remove_piece_at(&move_.from, &piece_to_move);
                        self.add_piece_at(
                            &move_.to,
                            &Piece::new(piece_to_move.player(), *promoted_to),
                        );
                    },
                }

                println!("{move_kind:?}");
                self.history.push(move_kind);
                self.update_mailbox();
                return Ok(());
            }
        }
    }

    /// This function assumes that the selected_move has already been validated as
    /// a legal move.
    fn classify_move(&self, piece_to_move: &Piece, selected_move: SelectedMove) -> UndoableMove {
        let move_ = selected_move.move_();

        let player_to_move = piece_to_move.player();
        let to = move_.to.as_u64();

        let en_passant_target = self.en_passant_target_square();
        if piece_to_move.kind() == PieceKind::Pawn
            && en_passant_target.is_some()
            && *en_passant_target.as_ref().unwrap() == move_.to
        {
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

            return UndoableMove::EnPassant {
                move_: selected_move.take_move(),
                captured_pawn_location: Location::try_from(real_pawn_location.0)
                    .expect(Location::failed_from_usize_message()),
            };
        }

        let to_location = Location::try_from(to).expect(Location::failed_from_usize_message());
        if let Some(captured_piece) = self.at(&to_location) {
            let to_rank = move_.to.rank();
            if piece_to_move.kind() == PieceKind::Pawn && (to_rank == Rank::One
                || to_rank == Rank::Eight)
            {
                let promotion_kind = selected_move.promotion_kind();
                return UndoableMove::CapturePromotion {
                    move_: selected_move.take_move(),
                    captured_piece: captured_piece,
                    promoted_to: promotion_kind
                        .expect("Promotion to have been validated by this point."),
                };
            }

            return UndoableMove::Capture {
                move_: selected_move.take_move(),
                captured_piece: captured_piece,
            };
        }

        let player = player_to_move.as_index();
        let from = move_.from.as_u64();

        match piece_to_move.kind() {
            PieceKind::Pawn => {
                if let Some(promotion) = selected_move.promotion_kind() {
                    return UndoableMove::Promotion {
                        move_: selected_move.take_move(),
                        promoted_to: promotion,
                    };
                }
            }
            PieceKind::King => {
                let from_loc =
                    Location::try_from(from).expect(Location::failed_from_usize_message());
                let to_loc = Location::try_from(to).expect(Location::failed_from_usize_message());

                if (from_loc.file().as_int() - to_loc.file().as_int()).abs() == 2 {
                    let castle_rank = match player {
                        white!() => Rank::One,
                        black!() => Rank::Eight,
                        _ => unreachable!(),
                    };

                    debug_assert!(castle_rank == to_loc.rank() && castle_rank == from_loc.rank());

                    if to_loc.file() == File::c {
                        return UndoableMove::Castles {
                            move_: selected_move.take_move(),
                            rook_move: Move {
                                from: Location::new(File::a, castle_rank),
                                to: Location::new(File::d, castle_rank),
                            },
                        };
                    } else if to_loc.file() == File::g {
                        return UndoableMove::Castles {
                            move_: selected_move.take_move(),
                            rook_move: Move {
                                from: Location::new(File::h, castle_rank),
                                to: Location::new(File::f, castle_rank),
                            },
                        };
                    } else {
                        panic!("Cannot castle at file {:?}", to_loc.file());
                    }
                }
            }
            PieceKind::Bishop | PieceKind::Knight | PieceKind::Rook | PieceKind::Queen => {}
        }

        return UndoableMove::Normal {
            move_: selected_move.take_move(),
        };
    }

    unsafe fn move_piece(&mut self, move_: &Move, piece: &Piece) {
        // FUTURE: optimize this by taking advantage of the piece being
        // the same for both operations
        unsafe { self.remove_piece_at(&move_.from, piece) };
        unsafe { self.add_piece_at(&move_.to, piece) };
    }

    unsafe fn move_piece_rev(&mut self, move_: &Move, piece: &Piece) {
        // FUTURE: optimize this by taking advantage of the piece being
        // the same for both operations
        unsafe { self.remove_piece_at(&move_.to, piece) };
        unsafe { self.add_piece_at(&move_.from, piece) };
    }

    unsafe fn remove_piece_at(&mut self, location: &Location, piece: &Piece) {
        self.xor_piece_at(location, piece)
    }

    unsafe fn add_piece_at(&mut self, location: &Location, piece: &Piece) {
        self.xor_piece_at(location, piece)
    }

    unsafe fn xor_piece_at(&mut self, location: &Location, piece: &Piece) {
        let player = piece.player().as_index();
        let location = location.as_u64();
        match piece.kind() {
            PieceKind::Pawn => self.pawns[player].0 ^= location,
            PieceKind::Knight => self.knights[player].0 ^= location,
            PieceKind::Bishop => self.bishops[player].0 ^= location,
            PieceKind::Rook => self.rooks[player].0 ^= location,
            PieceKind::Queen => self.queens[player].0 ^= location,
            PieceKind::King => self.kings[player].0 ^= location,
        }
    }

    /// Undoes the last move. This operation will fail if the
    /// undo stack is empty.
    pub fn undo(&mut self) -> Result<UndoableMove, ()> {
        match self.history.pop() {
            None => Err(()),
            Some(last_move) => {
                let player_to_undo = self.get_player_to_move();
                let piece_to_undo = self.at(&last_move.move_().to).expect(
                    "BOARD INTEGRITY: no piece found at 'to' location of move on top of undo stack",
                );
                assert!(player_to_undo == piece_to_undo.player(), "BOARD INTEGRITY: player to move and piece that is on top of undo stack mismatch");

                match &last_move {
                    UndoableMove::EnPassant {
                        move_,
                        captured_pawn_location,
                    } => unsafe {
                        self.move_piece_rev(move_, &piece_to_undo);

                        self.add_piece_at(
                            captured_pawn_location,
                            match player_to_undo.other_player() {
                                Player::White => &Self::WHITE_PAWN,
                                Player::Black => &Self::BLACK_PAWN,
                            },
                        );
                    },
                    UndoableMove::Castles { move_, rook_move } => {
                        let rook = Piece::new(piece_to_undo.player(), PieceKind::Rook);
                        unsafe {
                            self.move_piece_rev(move_, &piece_to_undo);
                            self.move_piece_rev(rook_move, &rook);
                        }
                    }
                    UndoableMove::Normal { move_ } => unsafe {
                        self.move_piece_rev(move_, &piece_to_undo);
                    },
                    UndoableMove::Capture {
                        move_,
                        captured_piece,
                    } => unsafe {
                        self.move_piece_rev(move_, &piece_to_undo);
                        self.add_piece_at(&move_.to, captured_piece);
                    },
                    UndoableMove::Promotion { move_, promoted_to } => {
                        assert!(*promoted_to == piece_to_undo.kind());

                        unsafe {
                            self.remove_piece_at(&move_.to, &piece_to_undo);
                            self.add_piece_at(
                                &move_.from,
                                &Piece::new(player_to_undo, PieceKind::Pawn),
                            )
                        }
                    }
                    UndoableMove::CapturePromotion {
                        move_,
                        captured_piece,
                        promoted_to,
                    } => unsafe {
                        assert!(*promoted_to == piece_to_undo.kind());

                        self.remove_piece_at(&move_.to, &piece_to_undo);
                        self.add_piece_at(&move_.to, captured_piece);
                        self.add_piece_at(&move_.from, &Piece::new(player_to_undo, PieceKind::Pawn));
                    },
                }

                self.update_mailbox();
                Ok(last_move)
            }
        }
    }
}

#[derive(Debug)]
pub enum MoveErr {
    IllegalMove,
    NoPieceAtFromLocation,
    IllegalPromotionPieceChoice,
    PromotionTargetNotPawn,
    MislabeledPromotion,
}

#[derive(Debug)]
pub enum UndoableMove {
    Promotion {
        move_: Move,
        // algebraic_notation: String,
        promoted_to: PieceKind,
    },
    EnPassant {
        move_: Move,
        // algebraic_notation: String,
        captured_pawn_location: Location,
    },
    Normal {
        move_: Move,
        // algebraic_notation: String,
    },
    Capture {
        move_: Move,
        // algebraic_notation: String,
        captured_piece: Piece,
    },
    CapturePromotion {
        move_: Move,
        captured_piece: Piece,
        promoted_to: PieceKind,
    },
    Castles {
        move_: Move,
        rook_move: Move,
    },
}

impl UndoableMove {
    pub fn move_(&self) -> &Move {
        match self {
            Self::Promotion { move_, .. } => move_,
            Self::EnPassant { move_, .. } => move_,
            Self::Normal { move_, .. } => move_,
            Self::Capture { move_, .. } => move_,
            Self::CapturePromotion { move_, .. } => move_,
            Self::Castles { move_, .. } => move_,
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}
