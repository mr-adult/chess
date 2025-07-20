mod acn_move_err;
use acn_move_err::AcnMoveErr;
mod move_err;
use iso_8859_1_encoder::Iso8859String;
use move_err::MoveErr;
mod undoable_move;
use streaming_iterator::StreamingIterator;
use undoable_move::UndoableMove;

use std::{
    str::FromStr,
    sync::{Arc, Mutex},
    thread::available_parallelism,
    vec::IntoIter,
};

use chess_common::{black, white, File, Location, Piece, PieceKind, Player, Rank};
use chess_parsers::{
    parse_algebraic_notation, parse_fen, BoardLayout, Check, FenErr, GameResult, NormalMove,
    ParsedGame, PieceLocations, PieceMove, PieceMoveKind,
};

use crate::{
    bitboard::BitBoard,
    legal_moves::{LegalKingMovesIterator, LegalMovesIterator},
    possible_moves::PossibleMovesIterator,
    IterativeDeepeningMovesIterator, Move, SelectedMove,
};

#[derive(Clone, Debug)]
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

impl Default for Board {
    fn default() -> Self {
        Self::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

impl FromStr for Board {
    type Err = FenErr;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let layout = parse_fen(str)?;
        Ok(Self::from(layout))
    }
}

impl From<BoardLayout> for Board {
    fn from(layout: BoardLayout) -> Self {
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

        for location in Location::all_locations() {
            if let Some(piece) = result.starting_position[&location] {
                result.get_bitboard_for(&piece).0 |= location.as_u64();
            }
        }

        result.update_mailbox();
        result
    }
}

impl Board {
    pub fn starting_position(&self) -> &BoardLayout {
        &self.starting_position
    }

    /// Calculates the material advantage of the current board position,
    /// assuming that pawns are worth 1 point, knights and bishops are worth 3 points,
    /// rooks are worth 5 points, and queens are worth 8 points.
    pub fn material_advantage(&self) -> i32 {
        let pawn_diff = self.pawns[white!()].bit_count() - self.pawns[black!()].bit_count();
        let knight_diff = self.knights[white!()].bit_count() - self.knights[black!()].bit_count();
        let bishop_diff = self.bishops[white!()].bit_count() - self.bishops[black!()].bit_count();
        let rook_diff = self.rooks[white!()].bit_count() - self.rooks[black!()].bit_count();
        let queen_diff = self.queens[white!()].bit_count() - self.queens[black!()].bit_count();

        return pawn_diff
            + knight_diff * 3
            + bishop_diff * 3
            + rook_diff * 5
            + queen_diff * 5
            + rook_diff * 8;
    }

    /// Loops through the bitboards and updates the mailbox bitboard.
    /// with the new piece locations. This should be called any time
    /// one of the pieces changes position or a piece is added/removed
    /// from the board.
    fn update_mailbox(&mut self) {
        let mut result = 0;
        for bitboard in self.all_bitboards() {
            result |= bitboard.0;
        }
        self.mailbox = BitBoard::new(result);
    }

    pub(crate) fn assert_board_integrity(&self) {
        #[cfg(not(debug_assertions))]
        return;

        for (i, bitboard_1) in self.all_bitboards().enumerate() {
            for (j, bitboard_2) in self.all_bitboards().enumerate() {
                if bitboard_1 as *const BitBoard == bitboard_2 as *const BitBoard {
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

    /// Retrieves all bitboards (except the mailbox bitboard)
    fn all_bitboards(&self) -> impl Iterator<Item = &BitBoard> {
        return self
            .pawns
            .iter()
            .chain(self.knights.iter())
            .chain(self.bishops.iter())
            .chain(self.rooks.iter())
            .chain(self.queens.iter())
            .chain(self.kings.iter());
    }

    /// Creates a mailbox bitboard for the specified player.
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

    /// Gets the player whose turn it currently is.
    pub fn player_to_move(&self) -> Player {
        if self.history.len() % 2 == 0 {
            return self.first_player_to_move;
        } else {
            return self.first_player_to_move.other_player();
        }
    }

    /// Gets whether or not the specified player can castle kingside.
    pub(crate) fn player_can_castle_kingside(&self, player: &Player) -> bool {
        match player {
            Player::White => {
                if !self.starting_position.white_can_castle_kingside() {
                    return false;
                }
            }
            Player::Black => {
                if !self.starting_position.black_can_castle_kingside() {
                    return false;
                }
            }
        }

        return !self
            .history
            .iter()
            .map(|undoable_move| undoable_move.move_())
            .flat_map(|move_| [&move_.from, &move_.to])
            .any(|loc| {
                *loc == Location::king_starting(player)
                    || *loc == Location::new(File::h, Rank::castle(player))
            });
    }

    /// Gets whether or not white can castle kingside.
    fn white_can_castle_kingside(&self) -> bool {
        self.player_can_castle_kingside(&Player::White)
    }

    /// Gets whether or not black can castle kingside.
    fn black_can_castle_kingside(&self) -> bool {
        self.player_can_castle_kingside(&Player::Black)
    }

    /// Gets whether or not the specified player can castle queenside.
    pub(crate) fn player_can_castle_queenside(&self, player: &Player) -> bool {
        match player {
            Player::White => {
                if !self.starting_position.white_can_castle_queenside() {
                    return false;
                }
            }
            Player::Black => {
                if !self.starting_position.black_can_castle_queenside() {
                    return false;
                }
            }
        }

        return !self
            .history
            .iter()
            .map(|undoable_move| undoable_move.move_())
            .flat_map(|move_| [&move_.from, &move_.to])
            .any(|loc| {
                *loc == Location::king_starting(player)
                    || *loc == Location::new(File::a, Rank::castle(player))
            });
    }

    /// Gets whether or not white can castle queenside.
    fn white_can_castle_queenside(&self) -> bool {
        self.player_can_castle_queenside(&Player::White)
    }

    /// Gets whether or not black can castle queenside.
    fn black_can_castle_queenside(&self) -> bool {
        self.player_can_castle_queenside(&Player::Black)
    }

    /// Gets the number of half-moves played in the current game as defined
    /// by Forsyth–Edwards Notation.
    pub fn half_moves_played(&self) -> u8 {
        let total = self.starting_position.half_move_counter() as usize + self.history.len();
        if total > u8::MAX as usize {
            return u8::MAX;
        }
        return total as u8;
    }

    /// Gets the number of full-moves played in the current game as defined
    /// by Forsyth–Edwards Notation.
    pub fn full_moves_played(&self) -> u8 {
        let history_len_u8 = self.history.len() as u8;
        let recorded_full_moves = match self.starting_position.player_to_move() {
            Player::White => history_len_u8 / 2,
            Player::Black => history_len_u8 / 2 + history_len_u8 % 2,
        };

        let total = recorded_full_moves as usize + self.history.len();
        if total > u8::MAX as usize {
            return u8::MAX;
        } else {
            return total as u8;
        }
    }

    /// Gets the current en-passant target square (if there is one).
    pub fn en_passant_target_square(&self) -> Option<Location> {
        // if a move has been made, refer to it.
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
            // if no move has been made, reference the initial board state's en-passant target.
        } else if let Some(location) = self.starting_position.en_passant_target_square() {
            return Some(location.clone());
        }

        return None;
    }

    /// Gets whether the current position is a check for the player whose turn
    /// it is.
    fn is_check(&self) -> bool {
        let player_to_move = self.player_to_move();
        let king_position = self.kings[player_to_move.as_index()].0;
        LegalKingMovesIterator::is_check(self, player_to_move, king_position)
    }

    /// Gets whether the current position is a checkmate for the player whose turn
    /// it is.
    pub fn is_check_mate(&self) -> bool {
        self.legal_moves().next().is_none() && self.is_check()
    }

    /// Gets whether the current position is stalemate.
    fn is_stale_mate(&self) -> bool {
        self.legal_moves().next().is_none() && !self.is_check()
    }

    /// Gets the piece at the specified location.
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

    /// Gets the list of historical moves in algebraic chess
    /// notation.
    pub fn get_move_history_acn(&self) -> Vec<PieceMove> {
        if self.history.is_empty() {
            return Vec::with_capacity(0);
        }

        // Clone the board in its initial state so we can replay the moves
        let mut temp_board = Self::from(self.starting_position.clone());
        let mut result = Vec::with_capacity(self.history.len());

        for undoable_move in self.history.iter() {
            let expect_piece_at = |loc: &Location| {
                temp_board.at(loc).unwrap_or_else(|| {
                    panic!(
                        "BOARD INTEGRITY: board history does not align with board state. {self:?}"
                    )
                })
            };

            let map_standard_move = |move_: &Move, is_capture: bool| {
                let moving_piece = expect_piece_at(&move_.from);

                let mut disambiguation_file = None;
                let mut disambiguation_rank = None;

                let conflicts = temp_board
                    .legal_moves()
                    .filter(|possible_move| {
                        let inner_move = possible_move.move_();
                        inner_move.to == move_.to && inner_move.from != move_.from
                    })
                    .filter(|potential_conflict| {
                        let conflict_piece = expect_piece_at(&potential_conflict.move_().from);
                        moving_piece == conflict_piece
                    })
                    .map(|possible_move| possible_move.take_move())
                    .collect::<Vec<_>>();

                if is_capture && moving_piece.kind() == PieceKind::Pawn {
                    disambiguation_file = Some(move_.from.file());
                }

                if !conflicts.is_empty() {
                    let move_from_file = move_.from.file();
                    let move_from_rank = move_.from.rank();

                    // if file is enough to disambiguate, just use that.
                    if !conflicts
                        .iter()
                        .any(|conflict| conflict.from.file() == move_.from.file())
                    {
                        disambiguation_file = Some(move_from_file);
                        // if rank is enough to disambiguate, just use that.
                    } else if !conflicts
                        .iter()
                        .any(|conflict| conflict.from.rank() == move_.from.rank())
                    {
                        disambiguation_rank = Some(move_from_rank);
                        // neither file nor rank is enough to disambiguate, so use both.
                    } else {
                        disambiguation_file = Some(move_from_file);
                        disambiguation_rank = Some(move_from_rank);
                    }
                }

                PieceMoveKind::Normal(NormalMove {
                    piece_kind: moving_piece.kind(),
                    destination: move_.to.clone(),
                    disambiguation_file,
                    disambiguation_rank,
                    is_capture,
                    promotion_kind: None,
                    move_suffix_annotations: Default::default(),
                })
            };

            let piece_move_kind = match &undoable_move {
                UndoableMove::EnPassant { move_, .. } | UndoableMove::Capture { move_, .. } => map_standard_move(move_, true),
                UndoableMove::Normal { move_ } => map_standard_move(move_, false),
                UndoableMove::Castles { move_, .. } => {
                    match move_.to.file() {
                        File::c => {
                            PieceMoveKind::CastleQueenside
                        },
                        File::g => {
                            PieceMoveKind::CastleKingside
                        },
                        _ => panic!("BOARD INTEGRITY: A move claimed to be a castles to a file other than c or g. {undoable_move:?}"),
                    }
                }
                UndoableMove::Promotion { move_, promoted_to } => {
                    PieceMoveKind::Normal(NormalMove {
                        piece_kind: PieceKind::Pawn,
                        destination: move_.to.clone(),
                        disambiguation_file: None,
                        disambiguation_rank: None,
                        is_capture: false,
                        promotion_kind: Some(*promoted_to),
                        move_suffix_annotations: Default::default(),
                    })
                }
                UndoableMove::CapturePromotion { move_, promoted_to, .. } => {
                    let to_file = move_.to.file().as_int();
                    let from_file = move_.from.file().as_int();

                    let mut disambiguation_file = None;
                    if let Ok(potential_disambiguation_file) = File::try_from(from_file - ((to_file - from_file) << 1 /* faster multiply by 2 */)) {
                        if temp_board.pawns[temp_board.player_to_move().as_index()].intersects_with_u64(Location::new(potential_disambiguation_file, move_.from.rank()).as_u64()) {
                            disambiguation_file = Some(potential_disambiguation_file);
                        }
                    }

                    PieceMoveKind::Normal(NormalMove {
                        piece_kind: PieceKind::Pawn,
                        destination: move_.to.clone(),
                        disambiguation_file: disambiguation_file,
                        disambiguation_rank: None, // rank is never ambiguous in a promotion
                        is_capture: true,
                        promotion_kind: Some(*promoted_to),
                        move_suffix_annotations: Default::default(),
                    })
                }
            };

            temp_board
                .make_move(undoable_move.into())
                .expect("BOARD INTEGRITY: a move from the history could not be replayed.");

            let check_status = if temp_board.is_check() {
                if temp_board.is_check_mate() {
                    Check::Mate
                } else {
                    Check::Check
                }
            } else {
                Check::None
            };

            result.push(PieceMove {
                check_kind: check_status,
                move_kind: piece_move_kind,
            });
        }

        result
    }

    pub fn legal_moves<'board>(&'board self) -> LegalMovesIterator<'board> {
        LegalMovesIterator::for_board(self)
    }

    pub fn possible_moves<'board>(&'board self) -> PossibleMovesIterator<'board> {
        PossibleMovesIterator::new(self.legal_moves())
    }

    pub fn iterative_deepening_bfs<'board>(
        &'board mut self,
        max_depth: usize,
    ) -> IterativeDeepeningMovesIterator<'board> {
        IterativeDeepeningMovesIterator::new(self, max_depth)
    }

    /// Runs a perft test for this board.
    /// * `depth` - The depth to search.
    /// * `max_dop` - The maximum degrees of parallelism.
    /// If 0 or 1, this will run single-threaded.
    /// If this value is greater than the available degrees of parallelism,
    /// it will be ignored and the available degrees of parallelism will
    /// be used instead.
    pub fn perft(&mut self, depth: usize, max_dop: usize) -> Vec<(PieceMove, usize)> {
        if depth == 0 {
            return Vec::with_capacity(0);
        }

        let possible_moves = self.possible_moves().collect::<Vec<_>>();
        let mut results = Vec::with_capacity(possible_moves.len());

        let mut dop = max_dop;
        if dop < 1 {
            dop = 1;
        }

        match available_parallelism() {
            Err(_) => { /* couldn't get the available parallelism, so just use the input value */ }
            Ok(available_parallel) => {
                let available_parallel = available_parallel.get();
                if dop > available_parallel {
                    dop = available_parallel;
                }
            }
        }

        let mut threads = Vec::with_capacity(dop);
        let work_queue = Arc::new(Mutex::new(possible_moves.into_iter()));

        let task = move |mut board: Board,
                         mut next: Option<SelectedMove>,
                         queue: Arc<Mutex<IntoIter<SelectedMove>>>| {
            let mut results = Vec::new();
            let mut had_previous_move = false;

            while let Some(move_) = next {
                if had_previous_move {
                    board.undo().unwrap();
                }

                board.make_move_unchecked(move_).unwrap();
                had_previous_move = true;
                let mut iter: IterativeDeepeningMovesIterator =
                    board.iterative_deepening_bfs(depth - 1);

                let mut total = 0_usize;
                let analyzed_move = iter.board().get_move_history_acn().last().unwrap().clone();
                while let Some(_) = iter.next() {
                    if iter.current_depth() < depth - 1 {
                        continue;
                    }

                    total += 1;
                }

                if depth == 1 {
                    total += 1;
                }

                results.push((analyzed_move, total));

                let mut lock = queue.lock().unwrap();
                next = lock.next();
                drop(lock);
            }
            results
        };

        // if we had >1 degree of parallelism, kick off the other threads.
        for _ in 0..dop - 1 {
            let board: Board = self.clone();
            let queue = work_queue.clone();

            let mut lock = queue.lock().unwrap();
            let next = lock.next();
            drop(lock);

            if next.is_some() {
                threads.push(std::thread::spawn(move || task(board, next, queue)));
            } else {
                break;
            }
        }

        let mut lock = work_queue.lock().unwrap();
        let start = lock.next();
        drop(lock);

        if start.is_some() {
            // start working through the queue on the main thread.
            for item in task(self.clone(), start, work_queue) {
                results.push(item);
            }
        }

        for handle in threads {
            for item in handle.join().unwrap() {
                results.push(item);
            }
        }

        results.sort_by(|(move_1, _), (move_2, _)| move_1.to_string().cmp(&move_2.to_string()));
        results
    }

    /// Makes a move where the move is passed in in algebraic chess notation
    pub fn make_move_acn(&mut self, acn: &str) -> Result<SelectedMove, AcnMoveErr> {
        if let Some(move_) = parse_algebraic_notation(acn.trim()) {
            let player_to_move = self.player_to_move();
            let selected_move = match move_.move_kind {
                PieceMoveKind::CastleKingside => {
                    // No need to validate its legality. make_move() will do that.
                    SelectedMove::Normal {
                        move_: match player_to_move {
                            Player::White => Move::WHITE_CASTLE_KINGSIDE,
                            Player::Black => Move::BLACK_CASTLE_KINGSIDE,
                        },
                    }
                }
                PieceMoveKind::CastleQueenside => {
                    // No need to validate its legality. make_move() will do that.
                    SelectedMove::Normal {
                        move_: match player_to_move {
                            Player::White => Move::WHITE_CASTLE_QUEENSIDE,
                            Player::Black => Move::BLACK_CASTLE_QUEENSIDE,
                        },
                    }
                }
                PieceMoveKind::Normal(normal_move_data) => {
                    let mut candidates = Vec::new();
                    for legal_move in self.legal_moves() {
                        let candidate_move = legal_move.move_();
                        if candidate_move.to != normal_move_data.destination {
                            continue;
                        }

                        if let Some(rank) = normal_move_data.disambiguation_rank {
                            if rank != candidate_move.from.rank() {
                                continue;
                            }
                        }

                        if let Some(file) = normal_move_data.disambiguation_file {
                            if file != candidate_move.from.file() {
                                continue;
                            }
                        }

                        let piece = self.at(&candidate_move.from).expect(
                            "BOARD INTEGRITY: mismatch between make_move and legal_moves logic",
                        );

                        if piece.player() != player_to_move
                            || normal_move_data.piece_kind != piece.kind()
                        {
                            continue;
                        }

                        candidates.push(legal_move);
                    }

                    if candidates.is_empty() {
                        return Err(AcnMoveErr::Move(MoveErr::IllegalMove));
                    }

                    if candidates.len() > 1 {
                        return Err(AcnMoveErr::AmbiguousMove);
                    }

                    let move_ = candidates.into_iter().next().unwrap();
                    match normal_move_data.promotion_kind {
                        None => SelectedMove::Normal {
                            move_: move_.move_().clone(),
                        },
                        Some(promotion_kind) => SelectedMove::Promotion {
                            move_: move_.move_().clone(),
                            promotion_kind,
                        },
                    }
                }
            };

            let selected_move_to_return = selected_move.clone();
            self.make_move(selected_move)?;

            match move_.check_kind {
                Check::Mate => {
                    if self.is_check_mate() {
                        return Ok(selected_move_to_return);
                    } else {
                        self.undo().expect("a move to be on the undo stack");
                        return Err(AcnMoveErr::CheckStateMismatch);
                    }
                }
                Check::Check => {
                    if self.is_check_mate() {
                        self.undo().expect("a move to be on the undo stack");
                        return Err(AcnMoveErr::CheckStateMismatch);
                    } else if self.is_check() {
                        return Ok(selected_move_to_return);
                    } else {
                        self.undo().expect("a move to be on the undo stack");
                        return Err(AcnMoveErr::CheckStateMismatch);
                    }
                }
                Check::None => {
                    if self.is_check() {
                        self.undo().expect("a move to be on the undo stack");
                        return Err(AcnMoveErr::CheckStateMismatch);
                    } else {
                        return Ok(selected_move_to_return);
                    }
                }
            }
        } else {
            return Err(AcnMoveErr::Acn);
        }
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

    /// Makes a move while skipping some of the more expensive validity checks.
    ///
    /// This function does not check if the move is legal.
    ///
    /// This function will still return an error if
    /// 1. the move's from location does not contain a piece
    /// 2. the piece being promoted is not a pawn
    /// 3. the promotion_kind is to a pawn to king
    /// as all of these checks are cheap.
    pub fn make_move_unchecked(&mut self, selected_move: SelectedMove) -> Result<(), MoveErr> {
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
                                Player::White => &Piece::WHITE_PAWN,
                                Player::Black => &Piece::BLACK_PAWN,
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
            if piece_to_move.kind() == PieceKind::Pawn
                && (to_rank == Rank::One || to_rank == Rank::Eight)
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
                    let castle_rank =
                        Rank::castle(&Player::try_from(player).expect("player to be valid"));
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
        let bitboard = self.get_bitboard_for(&piece);

        // Remove the piece at its old location
        bitboard.0 ^= move_.from.as_u64();
        // Add the piece at its new location
        bitboard.0 ^= move_.to.as_u64();
    }

    unsafe fn move_piece_rev(&mut self, move_: &Move, piece: &Piece) {
        let bitboard = self.get_bitboard_for(piece);

        // Remove the piece at its new location
        bitboard.0 ^= move_.to.as_u64();
        // Add the piece at its old location
        bitboard.0 ^= move_.from.as_u64();
    }

    unsafe fn remove_piece_at(&mut self, location: &Location, piece: &Piece) {
        self.xor_piece_at(location, piece)
    }

    unsafe fn add_piece_at(&mut self, location: &Location, piece: &Piece) {
        self.xor_piece_at(location, piece)
    }

    unsafe fn xor_piece_at(&mut self, location: &Location, piece: &Piece) {
        self.get_bitboard_for(piece).0 ^= location.as_u64();
    }

    fn get_bitboard_for(&mut self, piece: &Piece) -> &mut BitBoard {
        let player = piece.player().as_index();
        match piece.kind() {
            PieceKind::Pawn => &mut self.pawns[player],
            PieceKind::Knight => &mut self.knights[player],
            PieceKind::Bishop => &mut self.bishops[player],
            PieceKind::Rook => &mut self.rooks[player],
            PieceKind::Queen => &mut self.queens[player],
            PieceKind::King => &mut self.kings[player],
        }
    }

    /// Undoes the last move. This operation will fail if the
    /// undo stack is empty.
    pub fn undo(&mut self) -> Result<UndoableMove, ()> {
        match self.history.pop() {
            None => Err(()),
            Some(last_move) => {
                let player_to_undo = self.player_to_move();
                let piece_to_undo = self.at(&last_move.move_().to).expect(
                    "BOARD INTEGRITY: no piece found at 'to' location of move on top of undo stack",
                );

                if player_to_undo != piece_to_undo.player() {
                    let selected: SelectedMove = (&last_move).into();
                    self.make_move(selected).unwrap();
                    let move_acn = self
                        .get_move_history_acn()
                        .into_iter()
                        .map(|move_| move_.to_string())
                        .collect::<Vec<_>>()
                        .join(" ");

                    self.undo().ok();
                    panic!("BOARD INTEGRITY: player to move and piece that is on top of undo stack mismatch. Move history: {}", move_acn);
                }

                match &last_move {
                    UndoableMove::EnPassant {
                        move_,
                        captured_pawn_location,
                    } => unsafe {
                        self.move_piece_rev(move_, &piece_to_undo);

                        self.add_piece_at(
                            captured_pawn_location,
                            match player_to_undo.other_player() {
                                Player::White => &Piece::WHITE_PAWN,
                                Player::Black => &Piece::BLACK_PAWN,
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
                        self.add_piece_at(
                            &move_.from,
                            &Piece::new(player_to_undo, PieceKind::Pawn),
                        );
                    },
                }

                self.update_mailbox();
                Ok(last_move)
            }
        }
    }

    pub fn to_fen_string(&self) -> String {
        let layout: BoardLayout = self.into();
        layout.to_string()
    }

    pub fn to_pgn(&self) -> Iso8859String {
        let pgn: ParsedGame = self.into();
        (&pgn).into()
    }
}

impl Into<BoardLayout> for Board {
    fn into(self) -> BoardLayout {
        (&self).into()
    }
}

impl Into<BoardLayout> for &Board {
    fn into(self) -> BoardLayout {
        let piece_locations: PieceLocations = self.into();

        BoardLayout::new(
            piece_locations,
            self.player_to_move(),
            self.white_can_castle_kingside(),
            self.white_can_castle_queenside(),
            self.black_can_castle_kingside(),
            self.black_can_castle_queenside(),
            self.en_passant_target_square(),
            self.half_moves_played(),
            self.full_moves_played(),
        )
    }
}

impl Into<PieceLocations> for Board {
    fn into(self) -> PieceLocations {
        (&self).into()
    }
}

impl Into<PieceLocations> for &Board {
    fn into(self) -> PieceLocations {
        self.assert_board_integrity();
        let mut result = PieceLocations::default();

        for location in Location::all_locations() {
            result[&location] = self.at(&location);
        }

        result
    }
}

impl Into<ParsedGame> for &Board {
    fn into(self) -> ParsedGame {
        let result = if self.is_check_mate() {
            match self.player_to_move() {
                Player::White => GameResult::BlackWin,
                Player::Black => GameResult::WhiteWin,
            }
        } else if self.is_stale_mate() {
            GameResult::Draw
        } else {
            GameResult::Inconclusive
        };

        ParsedGame::new(Vec::new(), self.get_move_history_acn(), result).unwrap()
    }
}

pub mod perft_tests {
    use std::{collections::HashSet, io::Write, str::FromStr};

    use chess_common::{File, Location, PieceKind, Rank};
    use chess_parsers::PieceMove;

    use crate::{Board, Move, SelectedMove};

    #[allow(unused)]
    pub fn run_all() {
        // these all run single-threaded, so kick off threads for each.
        let mut threads = Vec::new();
        threads.push(std::thread::spawn(|| {
            starting_position_1();
            write_to_stdout("Starting position depth 1 passed.");
        }));
        threads.push(std::thread::spawn(|| {
            starting_position_2();
            write_to_stdout("Starting position depth 2 passed.");
        }));
        threads.push(std::thread::spawn(|| {
            starting_position_3();
            write_to_stdout("Starting position depth 3 passed.");
        }));
        threads.push(std::thread::spawn(|| {
            starting_position_4();
            write_to_stdout("Starting position depth 4 passed.");
        }));
        threads.push(std::thread::spawn(|| {
            starting_position_5();
            write_to_stdout("Starting position depth 5 passed.");
        }));
        threads.push(std::thread::spawn(|| {
            kiwipete_1();
            write_to_stdout("Kiwipete depth 1 passed.");
        }));
        threads.push(std::thread::spawn(|| {
            kiwipete_2();
            write_to_stdout("Kiwipete depth 2 passed.");
        }));
        threads.push(std::thread::spawn(|| {
            kiwipete_3();
            write_to_stdout("Kiwipete depth 3 passed.");
        }));
        threads.push(std::thread::spawn(|| {
            position3_1();
            write_to_stdout("Position 3 depth 1 passed.");
        }));
        threads.push(std::thread::spawn(|| {
            position3_2();
            write_to_stdout("Position 3 depth 2 passed.");
        }));
        threads.push(std::thread::spawn(|| {
            position3_3();
            write_to_stdout("Position 3 depth 3 passed.");
        }));
        threads.push(std::thread::spawn(|| {
            position3_4();
            write_to_stdout("Position 3 depth 4 passed.");
        }));
        threads.push(std::thread::spawn(|| {
            position3_5();
            write_to_stdout("Position 3 depth 5 passed.");
        }));
        for thread in threads {
            thread.join().unwrap();
        }

        // Everything past this runs in parallel, so run them one at a time.
        starting_position_6();
        write_to_stdout("Starting position depth 6 passed.");
        starting_position_7();
        write_to_stdout("Starting position depth 7 passed.");

        // Everything past this runs in parallel, so run them one at a time.
        kiwipete_4();
        write_to_stdout("Kiwipete depth 4 passed.");
        kiwipete_5();
        write_to_stdout("Kiwipete depth 5 passed.");

        // Everything past this runs in parallel, so run them one at a time.
        position3_6();
        write_to_stdout("Position 3 depth 6 passed.");
    }

    fn write_to_stdout(str: &str) {
        let mut stdout = std::io::stdout().lock();
        stdout.write(str.as_bytes()).unwrap();
        stdout.write(&[b'\n']).unwrap();
        stdout.flush().unwrap();
    }

    #[test]
    fn starting_position_1_test() {
        starting_position_1();
    }

    fn starting_position_1() {
        let mut board = Board::default();

        let expected_1 = parse_move_list(vec![
            ("a2a3", 1),
            ("b2b3", 1),
            ("c2c3", 1),
            ("d2d3", 1),
            ("e2e3", 1),
            ("f2f3", 1),
            ("g2g3", 1),
            ("h2h3", 1),
            ("a2a4", 1),
            ("b2b4", 1),
            ("c2c4", 1),
            ("d2d4", 1),
            ("e2e4", 1),
            ("f2f4", 1),
            ("g2g4", 1),
            ("h2h4", 1),
            ("b1a3", 1),
            ("b1c3", 1),
            ("g1f3", 1),
            ("g1h3", 1),
        ]);

        let perft = board.perft(1, 1);
        assert_perft_equality(&mut board, expected_1, perft);
    }

    #[test]
    fn starting_position_2_test() {
        starting_position_2();
    }

    fn starting_position_2() {
        let mut board = Board::default();

        let expected_2 = parse_move_list(vec![
            ("a2a3", 20),
            ("b2b3", 20),
            ("c2c3", 20),
            ("d2d3", 20),
            ("e2e3", 20),
            ("f2f3", 20),
            ("g2g3", 20),
            ("h2h3", 20),
            ("a2a4", 20),
            ("b2b4", 20),
            ("c2c4", 20),
            ("d2d4", 20),
            ("e2e4", 20),
            ("f2f4", 20),
            ("g2g4", 20),
            ("h2h4", 20),
            ("b1a3", 20),
            ("b1c3", 20),
            ("g1f3", 20),
            ("g1h3", 20),
        ]);
        let perft = board.perft(2, 1);
        assert_perft_equality(&mut board, expected_2, perft);
    }

    #[test]
    fn starting_position_3_test() {
        starting_position_3();
    }

    fn starting_position_3() {
        let mut board = Board::default();
        let expected_3 = parse_move_list(vec![
            ("a2a3", 380),
            ("b2b3", 420),
            ("c2c3", 420),
            ("d2d3", 539),
            ("e2e3", 599),
            ("f2f3", 380),
            ("g2g3", 420),
            ("h2h3", 380),
            ("a2a4", 420),
            ("b2b4", 421),
            ("c2c4", 441),
            ("d2d4", 560),
            ("e2e4", 600),
            ("f2f4", 401),
            ("g2g4", 421),
            ("h2h4", 420),
            ("b1a3", 400),
            ("b1c3", 440),
            ("g1f3", 440),
            ("g1h3", 400),
        ]);
        let perft = board.perft(3, 1);
        assert_perft_equality(&mut board, expected_3, perft);
    }

    #[test]
    fn starting_position_4_test() {
        starting_position_4();
    }

    fn starting_position_4() {
        let mut board = Board::default();
        let expected = parse_move_list(vec![
            ("a2a3", 8457),
            ("b2b3", 9345),
            ("c2c3", 9272),
            ("d2d3", 11959),
            ("e2e3", 13134),
            ("f2f3", 8457),
            ("g2g3", 9345),
            ("h2h3", 8457),
            ("a2a4", 9329),
            ("b2b4", 9332),
            ("c2c4", 9744),
            ("d2d4", 12435),
            ("e2e4", 13160),
            ("f2f4", 8929),
            ("g2g4", 9328),
            ("h2h4", 9329),
            ("b1a3", 8885),
            ("b1c3", 9755),
            ("g1f3", 9748),
            ("g1h3", 8881),
        ]);
        let actual = board.perft(4, 1);
        assert_perft_equality(&mut board, expected, actual)
    }

    #[test]
    fn starting_position_5_test() {
        starting_position_5();
    }

    fn starting_position_5() {
        let mut board = Board::default();
        let expected = parse_move_list(vec![
            ("a2a3", 181046),
            ("b2b3", 215255),
            ("c2c3", 222861),
            ("d2d3", 328511),
            ("e2e3", 402988),
            ("f2f3", 178889),
            ("g2g3", 217210),
            ("h2h3", 181044),
            ("a2a4", 217832),
            ("b2b4", 216145),
            ("c2c4", 240082),
            ("d2d4", 361790),
            ("e2e4", 405385),
            ("f2f4", 198473),
            ("g2g4", 214048),
            ("h2h4", 218829),
            ("b1a3", 198572),
            ("b1c3", 234656),
            ("g1f3", 233491),
            ("g1h3", 198502),
        ]);
        let actual = board.perft(5, 1);
        assert_perft_equality(&mut board, expected, actual);
    }

    // Not using this as a test for performance reasons
    fn starting_position_6() {
        let mut board = Board::default();
        let expected = parse_move_list(vec![
            ("a2a3", 4463267),
            ("b2b3", 5310358),
            ("c2c3", 5417640),
            ("d2d3", 8073082),
            ("e2e3", 9726018),
            ("f2f3", 4404141),
            ("g2g3", 5346260),
            ("h2h3", 4463070),
            ("a2a4", 5363555),
            ("b2b4", 5293555),
            ("c2c4", 5866666),
            ("d2d4", 8879566),
            ("e2e4", 9771632),
            ("f2f4", 4890429),
            ("g2g4", 5239875),
            ("h2h4", 5385554),
            ("b1a3", 4856835),
            ("b1c3", 5708064),
            ("g1f3", 5723523),
            ("g1h3", 4877234),
        ]);
        let actual = board.perft(
            6,
            std::thread::available_parallelism()
                .and_then(|available| Ok(available.get()))
                .unwrap_or(4),
        );
        assert_perft_equality(&mut board, expected, actual);
    }

    fn starting_position_7() {
        let mut board = Board::default();
        let expected = parse_move_list(vec![
            ("a2a3", 106743106),
            ("b2b3", 133233975),
            ("c2c3", 144074944),
            ("d2d3", 227598692),
            ("e2e3", 306138410),
            ("f2f3", 102021008),
            ("g2g3", 135987651),
            ("h2h3", 106678423),
            ("a2a4", 137077337),
            ("b2b4", 134087476),
            ("c2c4", 157756443),
            ("d2d4", 269605599),
            ("e2e4", 309478263),
            ("f2f4", 119614841),
            ("g2g4", 130293018),
            ("h2h4", 138495290),
            ("b1a3", 120142144),
            ("b1c3", 148527161),
            ("g1f3", 147678554),
            ("g1h3", 120669525),
        ]);
        let actual = board.perft(
            7,
            std::thread::available_parallelism()
                .and_then(|available| Ok(available.get()))
                .unwrap_or(4),
        );
        assert_perft_equality(&mut board, expected, actual);
    }

    const KIWIPETE_FEN: &'static str =
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0";

    #[test]
    fn kiwipete_1_test() {
        kiwipete_1();
    }

    fn kiwipete_1() {
        let mut board = Board::from_str(KIWIPETE_FEN).unwrap();

        let expected = parse_move_list(vec![
            ("a2a3", 1),
            ("b2b3", 1),
            ("g2g3", 1),
            ("d5d6", 1),
            ("a2a4", 1),
            ("g2g4", 1),
            ("g2h3", 1),
            ("d5e6", 1),
            ("c3b1", 1),
            ("c3d1", 1),
            ("c3a4", 1),
            ("c3b5", 1),
            ("e5d3", 1),
            ("e5c4", 1),
            ("e5g4", 1),
            ("e5c6", 1),
            ("e5g6", 1),
            ("e5d7", 1),
            ("e5f7", 1),
            ("d2c1", 1),
            ("d2e3", 1),
            ("d2f4", 1),
            ("d2g5", 1),
            ("d2h6", 1),
            ("e2d1", 1),
            ("e2f1", 1),
            ("e2d3", 1),
            ("e2c4", 1),
            ("e2b5", 1),
            ("e2a6", 1),
            ("a1b1", 1),
            ("a1c1", 1),
            ("a1d1", 1),
            ("h1f1", 1),
            ("h1g1", 1),
            ("f3d3", 1),
            ("f3e3", 1),
            ("f3g3", 1),
            ("f3h3", 1),
            ("f3f4", 1),
            ("f3g4", 1),
            ("f3f5", 1),
            ("f3h5", 1),
            ("f3f6", 1),
            ("e1d1", 1),
            ("e1f1", 1),
            ("e1g1", 1),
            ("e1c1", 1),
        ]);
        let actual = board.perft(1, 1);
        assert_perft_equality(&mut board, expected, actual);
    }

    #[test]
    fn kiwipete_2_test() {
        kiwipete_2();
    }

    fn kiwipete_2() {
        let mut board = Board::from_str(KIWIPETE_FEN).unwrap();
        let expected = parse_move_list(vec![
            ("a2a3", 44),
            ("b2b3", 42),
            ("g2g3", 42),
            ("d5d6", 41),
            ("a2a4", 44),
            ("g2g4", 42),
            ("g2h3", 43),
            ("d5e6", 46),
            ("c3b1", 42),
            ("c3d1", 42),
            ("c3a4", 42),
            ("c3b5", 39),
            ("e5d3", 43),
            ("e5c4", 42),
            ("e5g4", 44),
            ("e5c6", 41),
            ("e5g6", 42),
            ("e5d7", 45),
            ("e5f7", 44),
            ("d2c1", 43),
            ("d2e3", 43),
            ("d2f4", 43),
            ("d2g5", 42),
            ("d2h6", 41),
            ("e2d1", 44),
            ("e2f1", 44),
            ("e2d3", 42),
            ("e2c4", 41),
            ("e2b5", 39),
            ("e2a6", 36),
            ("a1b1", 43),
            ("a1c1", 43),
            ("a1d1", 43),
            ("h1f1", 43),
            ("h1g1", 43),
            ("f3d3", 42),
            ("f3e3", 43),
            ("f3g3", 43),
            ("f3h3", 43),
            ("f3f4", 43),
            ("f3g4", 43),
            ("f3f5", 45),
            ("f3h5", 43),
            ("f3f6", 39),
            ("e1d1", 43),
            ("e1f1", 43),
            ("e1g1", 43),
            ("e1c1", 43),
        ]);
        let actual = board.perft(2, 1);
        assert_perft_equality(&mut board, expected, actual);
    }

    #[test]
    fn kiwipete_3_test() {
        kiwipete_3();
    }

    fn kiwipete_3() {
        let mut board = Board::from_str(KIWIPETE_FEN).unwrap();
        let expected = parse_move_list(vec![
            ("a2a3", 2186),
            ("b2b3", 1964),
            ("g2g3", 1882),
            ("d5d6", 1991),
            ("a2a4", 2149),
            ("g2g4", 1843),
            ("g2h3", 1970),
            ("d5e6", 2241),
            ("c3b1", 2038),
            ("c3d1", 2040),
            ("c3a4", 2203),
            ("c3b5", 2138),
            ("e5d3", 1803),
            ("e5c4", 1880),
            ("e5g4", 1878),
            ("e5c6", 2027),
            ("e5g6", 1997),
            ("e5d7", 2124),
            ("e5f7", 2080),
            ("d2c1", 1963),
            ("d2e3", 2136),
            ("d2f4", 2000),
            ("d2g5", 2134),
            ("d2h6", 2019),
            ("e2d1", 1733),
            ("e2f1", 2060),
            ("e2d3", 2050),
            ("e2c4", 2082),
            ("e2b5", 2057),
            ("e2a6", 1907),
            ("a1b1", 1969),
            ("a1c1", 1968),
            ("a1d1", 1885),
            ("h1f1", 1929),
            ("h1g1", 2013),
            ("f3d3", 2005),
            ("f3e3", 2174),
            ("f3g3", 2214),
            ("f3h3", 2360),
            ("f3f4", 2132),
            ("f3g4", 2169),
            ("f3f5", 2396),
            ("f3h5", 2267),
            ("f3f6", 2111),
            ("e1d1", 1894),
            ("e1f1", 1855),
            ("e1g1", 2059),
            ("e1c1", 1887),
        ]);
        let actual = board.perft(3, 1);
        assert_perft_equality(&mut board, expected, actual);
    }

    fn kiwipete_4() {
        let mut board = Board::from_str(KIWIPETE_FEN).unwrap();
        let expected = parse_move_list(vec![
            ("a2a3", 94405),
            ("b2b3", 81066),
            ("g2g3", 77468),
            ("d5d6", 79551),
            ("a2a4", 90978),
            ("g2g4", 75677),
            ("g2h3", 82759),
            ("d5e6", 97464),
            ("c3b1", 84773),
            ("c3d1", 84782),
            ("c3a4", 91447),
            ("c3b5", 81498),
            ("e5d3", 77431),
            ("e5c4", 77752),
            ("e5g4", 79912),
            ("e5c6", 83885),
            ("e5g6", 83866),
            ("e5d7", 93913),
            ("e5f7", 88799),
            ("d2c1", 83037),
            ("d2e3", 90274),
            ("d2f4", 84869),
            ("d2g5", 87951),
            ("d2h6", 82323),
            ("e2d1", 74963),
            ("e2f1", 88728),
            ("e2d3", 85119),
            ("e2c4", 84835),
            ("e2b5", 79739),
            ("e2a6", 69334),
            ("a1b1", 83348),
            ("a1c1", 83263),
            ("a1d1", 79695),
            ("h1f1", 81563),
            ("h1g1", 84876),
            ("f3d3", 83727),
            ("f3e3", 92505),
            ("f3g3", 94461),
            ("f3h3", 98524),
            ("f3f4", 90488),
            ("f3g4", 92037),
            ("f3f5", 104992),
            ("f3h5", 95034),
            ("f3f6", 77838),
            ("e1d1", 79989),
            ("e1f1", 77887),
            ("e1g1", 86975),
            ("e1c1", 79803),
        ]);
        let actual = board.perft(
            4,
            std::thread::available_parallelism()
                .and_then(|available| Ok(available.get()))
                .unwrap_or(4),
        );
        assert_perft_equality(&mut board, expected, actual);
    }

    fn kiwipete_5() {
        let mut board = Board::from_str(KIWIPETE_FEN).unwrap();
        let expected = parse_move_list(vec![
            ("a2a3", 4627439),
            ("b2b3", 3768824),
            ("g2g3", 3472039),
            ("d5d6", 3835265),
            ("a2a4", 4387586),
            ("g2g4", 3338154),
            ("g2h3", 3819456),
            ("d5e6", 4727437),
            ("c3b1", 3996171),
            ("c3d1", 3995761),
            ("c3a4", 4628497),
            ("c3b5", 4317482),
            ("e5d3", 3288812),
            ("e5c4", 3494887),
            ("e5g4", 3415992),
            ("e5c6", 4083458),
            ("e5g6", 3949417),
            ("e5d7", 4404043),
            ("e5f7", 4164923),
            ("d2c1", 3793390),
            ("d2e3", 4407041),
            ("d2f4", 3941257),
            ("d2g5", 4370915),
            ("d2h6", 3967365),
            ("e2d1", 3074219),
            ("e2f1", 4095479),
            ("e2d3", 4066966),
            ("e2c4", 4182989),
            ("e2b5", 4032348),
            ("e2a6", 3553501),
            ("a1b1", 3827454),
            ("a1c1", 3814203),
            ("a1d1", 3568344),
            ("h1f1", 3685756),
            ("h1g1", 3989454),
            ("f3d3", 3949570),
            ("f3e3", 4477772),
            ("f3g3", 4669768),
            ("f3h3", 5067173),
            ("f3f4", 4327936),
            ("f3g4", 4514010),
            ("f3f5", 5271134),
            ("f3h5", 4743335),
            ("f3f6", 3975992),
            ("e1d1", 3559113),
            ("e1f1", 3377351),
            ("e1g1", 4119629),
            ("e1c1", 3551583),
        ]);
        let actual = board.perft(
            5,
            std::thread::available_parallelism()
                .and_then(|available| Ok(available.get()))
                .unwrap_or(4),
        );
        assert_perft_equality(&mut board, expected, actual);
    }

    const POSITION3FEN: &'static str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 0";

    #[test]
    fn position3_1_test() {
        position3_1();
    }

    fn position3_1() {
        let mut board = Board::from_str(POSITION3FEN).unwrap();
        let expected = parse_move_list(vec![
            ("e2e3", 1),
            ("g2g3", 1),
            ("a5a6", 1),
            ("e2e4", 1),
            ("g2g4", 1),
            ("b4b1", 1),
            ("b4b2", 1),
            ("b4b3", 1),
            ("b4a4", 1),
            ("b4c4", 1),
            ("b4d4", 1),
            ("b4e4", 1),
            ("b4f4", 1),
            ("a5a4", 1),
        ]);
        let actual = board.perft(1, 1);
        assert_perft_equality(&mut board, expected, actual);
    }

    #[test]
    fn position3_2_test() {
        position3_2();
    }

    fn position3_2() {
        let mut board = Board::from_str(&POSITION3FEN).unwrap();
        let expected = parse_move_list(vec![
            ("e2e3", 15),
            ("g2g3", 4),
            ("a5a6", 15),
            ("e2e4", 16),
            ("g2g4", 17),
            ("b4b1", 16),
            ("b4b2", 16),
            ("b4b3", 15),
            ("b4a4", 15),
            ("b4c4", 15),
            ("b4d4", 15),
            ("b4e4", 15),
            ("b4f4", 2),
            ("a5a4", 15),
        ]);
        let actual = board.perft(2, 1);
        assert_perft_equality(&mut board, expected, actual);
    }

    #[test]
    fn position3_3_test() {
        position3_3();
    }

    fn position3_3() {
        let mut board = Board::from_str(POSITION3FEN).unwrap();
        let expected = parse_move_list(vec![
            ("e2e3", 205),
            ("g2g3", 54),
            ("a5a6", 240),
            ("e2e4", 177),
            ("g2g4", 226),
            ("b4b1", 265),
            ("b4b2", 205),
            ("b4b3", 248),
            ("b4a4", 202),
            ("b4c4", 254),
            ("b4d4", 243),
            ("b4e4", 228),
            ("b4f4", 41),
            ("a5a4", 224),
        ]);
        let actual = board.perft(3, 1);
        assert_perft_equality(&mut board, expected, actual);
    }

    #[test]
    fn position3_4_test() {
        position3_4();
    }
    fn position3_4() {
        let mut board = Board::from_str(POSITION3FEN).unwrap();
        let expected = parse_move_list(vec![
            ("e2e3", 3107),
            ("g2g3", 1014),
            ("a5a6", 3653),
            ("e2e4", 2748),
            ("g2g4", 3702),
            ("b4b1", 4199),
            ("b4b2", 3328),
            ("b4b3", 3658),
            ("b4a4", 3019),
            ("b4c4", 3797),
            ("b4d4", 3622),
            ("b4e4", 3391),
            ("b4f4", 606),
            ("a5a4", 3394),
        ]);
        let actual = board.perft(4, 1);
        assert_perft_equality(&mut board, expected, actual);
    }

    #[test]
    fn position3_5_test() {
        position3_5();
    }

    fn position3_5() {
        let mut board = Board::from_str(POSITION3FEN).unwrap();
        let expected = parse_move_list(vec![
            ("e2e3", 45326),
            ("g2g3", 14747),
            ("a5a6", 59028),
            ("e2e4", 36889),
            ("g2g4", 53895),
            ("b4b1", 69665),
            ("b4b2", 48498),
            ("b4b3", 59719),
            ("b4a4", 45591),
            ("b4c4", 63781),
            ("b4d4", 59574),
            ("b4e4", 54192),
            ("b4f4", 10776),
            ("a5a4", 52943),
        ]);
        let actual = board.perft(5, 1);
        assert_perft_equality(&mut board, expected, actual);
    }

    fn position3_6() {
        let mut board = Board::from_str(POSITION3FEN).unwrap();
        let expected = parse_move_list(vec![
            ("e2e3", 745505),
            ("g2g3", 271220),
            ("a5a6", 968724),
            ("e2e4", 597519),
            ("g2g4", 892781),
            ("b4b1", 1160678),
            ("b4b2", 818501),
            ("b4b3", 941129),
            ("b4a4", 745667),
            ("b4c4", 1027199),
            ("b4d4", 957108),
            ("b4e4", 860971),
            ("b4f4", 174919),
            ("a5a4", 868162),
        ]);
        let actual = board.perft(
            6,
            std::thread::available_parallelism()
                .and_then(|available| Ok(available.get()))
                .unwrap_or(4),
        );
        assert_perft_equality(&mut board, expected, actual);
    }

    fn parse_move_list(moves: Vec<(&str, usize)>) -> Vec<(SelectedMove, usize)> {
        moves
            .into_iter()
            .map(|move_| (parse_move(move_.0), move_.1))
            .collect()
    }

    fn parse_move(move_: &str) -> SelectedMove {
        let mut chars = move_.chars();
        let move_ = Move {
            from: Location::new(
                File::try_from(chars.next().unwrap()).unwrap(),
                Rank::try_from(chars.next().unwrap()).unwrap(),
            ),
            to: Location::new(
                File::try_from(chars.next().unwrap()).unwrap(),
                Rank::try_from(chars.next().unwrap()).unwrap(),
            ),
        };

        if let Some('=') = chars.next() {
            SelectedMove::Promotion {
                move_,
                promotion_kind: PieceKind::try_from(chars.next().unwrap()).unwrap(),
            }
        } else {
            SelectedMove::Normal { move_ }
        }
    }

    fn assert_perft_equality(
        board_in_starting_position: &mut Board,
        expected: Vec<(SelectedMove, usize)>,
        actual: Vec<(PieceMove, usize)>,
    ) {
        let expected_acn = expected
            .into_iter()
            .map(|expected_perft| {
                board_in_starting_position
                    .make_move(expected_perft.0)
                    .unwrap();
                let result = (
                    board_in_starting_position
                        .get_move_history_acn()
                        .pop()
                        .unwrap()
                        .to_string(),
                    expected_perft.1,
                );
                board_in_starting_position.undo().unwrap();
                result
            })
            .collect::<HashSet<_>>();

        let actual_acn = actual
            .into_iter()
            .map(|actual_perft| (actual_perft.0.to_string(), actual_perft.1))
            .collect::<HashSet<_>>();

        for expected in expected_acn.iter() {
            match actual_acn.get(&expected) {
                None => panic!("Expected to find move: {}, but did not.", expected.0),
                Some(actual) => {
                    assert_eq!(
                        expected.1, actual.1,
                        "Mismatch for move {}. Expected: {}, but got {}",
                        expected.0, expected.1, actual.1
                    );
                }
            }
        }

        for actual in actual_acn {
            if !expected_acn.contains(&actual) {
                panic!("Did not expect to find move {}, but did.", actual.0);
            }
        }
    }
}
