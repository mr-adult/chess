mod acn_move_err;
use acn_move_err::AcnMoveErr;
mod move_err;
use iso_8859_1_encoder::Iso8859String;
use move_err::MoveErr;
mod undoable_move;
use streaming_iterator::StreamingIterator;
use undoable_move::UndoableMove;

use std::{os::windows::thread, str::FromStr, sync::Mutex};

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
            Player::Black => self.black_can_castle_kingside(),
            Player::White => self.white_can_castle_kingside(),
        }
    }

    /// Gets whether or not white can castle kingside.
    fn white_can_castle_kingside(&self) -> bool {
        if !self.starting_position.white_can_castle_kingside() {
            return false;
        }

        return !self.history.iter().any(|undoable_move| {
            let move_ = undoable_move.move_();
            move_.from == Location::king_starting(&Player::White)
                || move_.from == Location::new(File::h, Rank::castle(&Player::White))
        });
    }

    /// Gets whether or not black can castle kingside.
    fn black_can_castle_kingside(&self) -> bool {
        if !self.starting_position.black_can_castle_kingside() {
            return false;
        }

        return !self.history.iter().any(|undoable_move| {
            let move_from = &undoable_move.move_().from;
            *move_from == Location::king_starting(&Player::Black)
                || *move_from == Location::new(File::h, Rank::castle(&Player::Black))
        });
    }

    /// Gets whether or not the specified player can castle queenside.
    pub(crate) fn player_can_castle_queenside(&self, player: &Player) -> bool {
        match player {
            Player::Black => self.black_can_castle_queenside(),
            Player::White => self.white_can_castle_queenside(),
        }
    }

    /// Gets whether or not white can castle queenside.
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

    /// Gets whether or not black can castle queenside.
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

    pub fn perft(&mut self, depth: usize) -> Vec<(PieceMove, usize)> {
        if depth == 0 {
            return Vec::with_capacity(0);
        }

        let possible_moves = self.possible_moves().collect::<Vec<_>>();
        let mut results = Vec::with_capacity(possible_moves.len());
        let mut threads = Vec::with_capacity(possible_moves.len());

        for move_ in possible_moves {
            let mut board = self.clone();
            board.make_move_unchecked(move_).unwrap();
            threads.push(std::thread::spawn(move || {
                let mut iter = board.iterative_deepening_bfs(depth - 1);

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

                (analyzed_move, total)
            }));
        }

        for handle in threads {
            results.push(handle.join().unwrap());
        }

        results.sort_by(|(move_1, _), (move_2, _)| move_1.to_string().cmp(&move_2.to_string()));
        results
    }

    /// Makes a move where the move is passed in in algebraic chess notation
    pub fn make_move_acn(&mut self, acn: &str) -> Result<(), AcnMoveErr> {
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

            self.make_move(selected_move)?;

            match move_.check_kind {
                Check::Mate => {
                    if self.is_check_mate() {
                        return Ok(());
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
                        return Ok(());
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
                        return Ok(());
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
