use std::{
    array::{from_fn, IntoIter},
    fmt::Display,
    iter,
    ops::{Index, IndexMut},
    str::FromStr,
};

use chess_common::{black, white, File, Location, PieceKind, Player, Rank};
use chess_parsers::{parse_fen, BoardLayout, FenErr};

use crate::{
    bitboard::BitBoard,
    chess_move::{IllegalMove, KingMoveKind, MoveKind, PawnMoveKind},
    piece::Piece,
    Move,
};

use self::shifts::{down_left, down_right, up_left, up_right};

pub(crate) struct ErgonomicBoard {
    pieces: [[Option<Piece>; 8]; 8],
}

impl ErgonomicBoard {
    fn new() -> Self {
        Self {
            pieces: from_fn(|_| from_fn(|_| None)),
        }
    }
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
pub(crate) struct Board {
    starting_position: BoardLayout,
    pawns: [BitBoard; 2],
    knights: [BitBoard; 2],
    bishops: [BitBoard; 2],
    rooks: [BitBoard; 2],
    queens: [BitBoard; 2],
    kings: [BitBoard; 2],
    /// A bitboard that represents occupied and unoccupied board squares
    mailbox: BitBoard,
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
                    Some((player, piece_kind)) => match piece_kind {
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
                    },
                }
            }
        }

        result.update_mailbox();
        Ok(result)
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
    fn assert_board_integrity(&self) {
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
                    panic!(
                        "Found conflicting bitboards at indexes {i} and {j}. {:?}",
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

    fn get_player_to_move(&self) -> Player {
        if self.history.len() & 1 == 0 {
            return self.first_player_to_move;
        }

        match self.first_player_to_move {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }

    fn make_move(&mut self, move_: Move) -> Result<(), IllegalMove> {
        if let Err(err) = self.classify_move(&move_) {
            return Err(err);
        }

        todo!()
    }

    fn make_move_unchecked(&mut self, move_: Move) -> Result<(), IllegalMove> {
        todo!();
    }

    fn is_legal_move(&self, move_: &Move) -> bool {
        self.classify_move(&move_).is_ok()
    }

    fn classify_move(&self, move_: &Move) -> Result<MoveKind, IllegalMove> {
        if move_.to == move_.from {
            return Ok(MoveKind::NonMove);
        }

        let from_bits = move_.from.as_u64();
        let to_bits = move_.to.as_u64();

        if self.mailbox.0 & from_bits == 0 {
            return Err(IllegalMove::NoPieceAtFromLocation);
        }

        match self.get_player_to_move() {
            Player::White => {
                let pawn_square = self.pawns[white!()].0 & from_bits;
                if pawn_square != 0 {
                    let single_push = pawn_square << 8;
                    debug_assert!(
                        single_push != 0,
                        "If this fails, it means we miscalculated a promotion"
                    );

                    // push square is occupied
                    if (self.mailbox.0 & single_push) != 0 {
                        return Err(IllegalMove::IllegalMove);
                    }

                    // If the pawn is on rank 2, check for double push
                    if (pawn_square | Rank::two_bit_filter()) != 0 {
                        let double_push = single_push << 8;
                        if double_push == to_bits {
                            // double push square is occupied
                            if (self.mailbox.0 | double_push) != 0 {
                                return Err(IllegalMove::IllegalMove);
                            } else {
                                // 1 is horizontal from square, 7 and 9 are diagonals and 8 is vertical
                                for shift in [1, 7, 8, 9] {}
                                if self.causes_check_for_white(move_) {
                                    return Err(IllegalMove::Check);
                                }
                                return Ok(MoveKind::Normal);
                            }
                        }
                    }

                    for shift in [7, 9] {
                        let capture_square = pawn_square << shift;
                        if (capture_square | self.mailbox.0) != 0 {
                            if self.causes_check_for_white(move_) {
                                return Err(IllegalMove::Check);
                            }
                            return Ok(MoveKind::Pawn(PawnMoveKind::Capture));
                        }
                    }

                    return Err(IllegalMove::IllegalMove);
                }

                let knight_square = self.knights[white!()].0 & from_bits;
                if self.knights[white!()].0 & from_bits != 0 {
                    todo!()
                } else if self.bishops[white!()].0 & from_bits != 0 {
                    todo!()
                } else if self.rooks[white!()].0 & from_bits != 0 {
                    todo!()
                } else if self.queens[white!()].0 & from_bits != 0 {
                    todo!()
                } else if self.kings[white!()].0 & from_bits != 0 {
                    todo!()
                } else {
                    return Err(IllegalMove::PieceOwnedByOtherPlayer);
                }
            }
            Player::Black => {
                if self.pawns[black!()].0 & from_bits != 0 {
                    debug_assert!(
                        ((self.pawns[black!()].0 & from_bits) >> 8) != 0,
                        "If this fails, it means we miscalculated a promotion"
                    );
                    todo!()
                } else if self.knights[black!()].0 & from_bits != 0 {
                    todo!()
                } else if self.bishops[black!()].0 & from_bits != 0 {
                    todo!()
                } else if self.rooks[black!()].0 & from_bits != 0 {
                    todo!()
                } else if self.queens[black!()].0 & from_bits != 0 {
                    todo!()
                } else if self.kings[black!()].0 & from_bits != 0 {
                    todo!()
                } else {
                    return Err(IllegalMove::PieceOwnedByOtherPlayer);
                }
            }
        }
    }

    fn causes_check_for_white(&self, move_: &Move) -> bool {
        let mut ray_casters: [u64; 8] = std::array::from_fn(|_| move_.from.as_u64());
        let mut shift_direction = 0;
        let mut shift_for_check = 0;
        let mut could_cause_check = false;
        for _ in 0..8 {
            // 1 is horizontal from square, 7 and 9 are diagonals and 8 is vertical
            for (i, shift) in [1, 7, 8, 9].into_iter().enumerate() {
                ray_casters[i] = ray_casters[i] << shift;
                if (ray_casters[i] & self.kings[white!()].0) != 0 {
                    shift_for_check = shift;
                    shift_direction = -1;
                    could_cause_check = true;
                    break;
                }
            }
            if could_cause_check {
                break;
            }

            for (i, shift) in [1, 7, 8, 9].into_iter().enumerate() {
                let index = 4 + i;
                ray_casters[index] = ray_casters[index] >> shift;
                if (ray_casters[index] & self.kings[white!()].0) != 0 {
                    shift_for_check = shift;
                    shift_direction = 1;
                    could_cause_check = true;
                    break;
                }
            }
            if could_cause_check {
                break;
            }
        }

        if !could_cause_check {
            return false;
        }
        todo!();
    }

    fn at(&self, location: Location) -> Option<Piece> {
        let location_bits = location.as_u64();

        // pawns are most common, so check them first
        if self.pawns[white!()].0 & location_bits != 0 {
            return Some(Piece::new(PieceKind::Pawn, Player::White));
        }

        if self.pawns[black!()].0 & location_bits != 0 {
            return Some(Piece::new(PieceKind::Pawn, Player::Black));
        }

        // Rooks are equally as common as bishops and knights, but tend
        // to be more prevalent in the mid to end game, so check them
        // before bishops/knights
        if self.rooks[white!()].0 & location_bits != 0 {
            return Some(Piece::new(PieceKind::Rook, Player::White));
        }

        if self.rooks[black!()].0 & location_bits != 0 {
            return Some(Piece::new(PieceKind::Rook, Player::Black));
        }

        if self.knights[white!()].0 & location_bits != 0 {
            return Some(Piece::new(PieceKind::Knight, Player::White));
        }

        if self.knights[black!()].0 & location_bits != 0 {
            return Some(Piece::new(PieceKind::Knight, Player::White));
        }

        if self.bishops[white!()].0 & location_bits != 0 {
            return Some(Piece::new(PieceKind::Bishop, Player::White));
        }

        // Only 1 queen/king per side, so check them last
        if self.bishops[black!()].0 & location_bits != 0 {
            return Some(Piece::new(PieceKind::Bishop, Player::Black));
        }

        if self.queens[white!()].0 & location_bits != 0 {
            return Some(Piece::new(PieceKind::Queen, Player::White));
        }

        if self.queens[black!()].0 & location_bits != 0 {
            return Some(Piece::new(PieceKind::Queen, Player::Black));
        }

        if self.kings[white!()].0 & location_bits != 0 {
            return Some(Piece::new(PieceKind::King, Player::White));
        }

        if self.kings[black!()].0 & location_bits != 0 {
            return Some(Piece::new(PieceKind::King, Player::Black));
        }

        return None;
    }

    fn into_ergo_board(&self) -> ErgonomicBoard {
        self.assert_board_integrity();
        let mut result = ErgonomicBoard::new();

        for rank in Rank::all_ranks_ascending() {
            for file in File::all_files_ascending() {
                let location = Location::new(file, rank);
                let location_bits = location.as_u64();

                result[location] = self.at(location);
            }
        }

        result
    }

    fn legal_moves<'a>(&'a self) -> impl Iterator<Item = Move> + 'a {
        LegalMovesIterator::for_board(self)
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        let ergo_board: ErgonomicBoard = self.into_ergo_board();

        for rank in Rank::all_ranks_ascending().rev() {
            for file in File::all_files_ascending() {
                let location = Location::new(file, rank);

                match &ergo_board[location] {
                    None => result.push_str(" * "),
                    Some(piece) => {
                        result.push(piece.player.as_char());
                        result.push(piece.kind.as_char());
                        result.push(' ');
                    }
                }
            }
            result.push('\n');
        }

        f.write_str(&result)
    }
}

struct LegalMovesIterator<'board> {
    board: &'board Board,
    player: Player,
    is_check: Option<bool>,
    pawn_moves_iterator: Option<LegalPawnMovesIterator<'board>>,
    knight_moves_iterator: Option<LegalKnightMovesIterator<'board>>,
    bishop_moves_iterator: Option<LegalBishopMovesIterator<'board>>,
    rook_moves_iterator: Option<LegalRookMovesIterator<'board>>,
    queen_moves_iterator: Option<LegalQueenMovesIterator<'board>>,
    king_moves_iterator: LegalKingMovesIterator<'board>,
    king_moves_iterator_finished: bool,
    check_blocking_squares: Option<[Option<Location>; 8]>,
}

impl<'board> LegalMovesIterator<'board> {
    fn for_board(board: &'board Board) -> Self {
        let player_to_move = board.get_player_to_move();
        Self {
            board,
            player: player_to_move,
            is_check: None,
            pawn_moves_iterator: Some(LegalPawnMovesIterator::new(board)),
            knight_moves_iterator: Some(LegalKnightMovesIterator { board }),
            bishop_moves_iterator: Some(LegalBishopMovesIterator { board }),
            rook_moves_iterator: Some(LegalRookMovesIterator { board }),
            queen_moves_iterator: Some(LegalQueenMovesIterator { board }),
            king_moves_iterator: LegalKingMovesIterator::new(board, player_to_move),
            king_moves_iterator_finished: false,
            check_blocking_squares: None,
        }
    }

    fn for_piece(board: &'board Board, piece: Piece) -> Self {
        Self {
            board,
            player: piece.player,
            is_check: None,
            pawn_moves_iterator: Some(LegalPawnMovesIterator::new(board)),
            knight_moves_iterator: Some(LegalKnightMovesIterator { board }),
            bishop_moves_iterator: Some(LegalBishopMovesIterator { board }),
            rook_moves_iterator: Some(LegalRookMovesIterator { board }),
            queen_moves_iterator: Some(LegalQueenMovesIterator { board }),
            king_moves_iterator: LegalKingMovesIterator::new(&board, piece.player),
            // We only want to iterate through king moves if the target piece is a king
            king_moves_iterator_finished: piece.kind != PieceKind::King,
            check_blocking_squares: None,
        }
    }

    /// This function calculates all squares where if the piece on
    /// that square moves, it will result in a check. The rules for
    /// this are as follows:
    /// 1. Draw 8 vectors out from the king: one per cardinal direction
    /// 2. If the vector hits the edge of the board, no check opportunities
    /// 3. If the vector hits 2 friendly pieces first, no check opportunities
    /// 4. If the vector hits an enemy piece, it is already a check and we do
    /// not include it
    /// 5. If the vector hits a friendly piece then an enemy piece,
    /// moving the friendly piece will result in a check. These moves
    /// are included.
    fn calculate_check_blocking_squares(&mut self) -> () {
        let white_piece_positions = self.board.pawns[white!()].0
            | self.board.knights[white!()].0
            | self.board.bishops[white!()].0
            | self.board.rooks[white!()].0
            | self.board.queens[white!()].0
            | self.board.kings[white!()].0;

        let black_piece_positions = self.board.pawns[black!()].0
            | self.board.knights[black!()].0
            | self.board.bishops[black!()].0
            | self.board.rooks[black!()].0
            | self.board.queens[black!()].0
            | self.board.kings[black!()].0;

        let defending_piece_mailbox;
        let attacking_piece_mailbox;
        let attacking_player;
        match self.player {
            Player::White => {
                defending_piece_mailbox = white_piece_positions;
                attacking_piece_mailbox = black_piece_positions;
                attacking_player = Player::Black.as_index();
            }
            Player::Black => {
                defending_piece_mailbox = black_piece_positions;
                attacking_piece_mailbox = white_piece_positions;
                attacking_player = Player::White.as_index();
            }
        }

        let king_bitboard = match self.player {
            Player::White => self.board.kings[white!()].0,
            Player::Black => self.board.kings[black!()].0,
        };
    }
}

impl<'board> Iterator for LegalMovesIterator<'board> {
    type Item = Move;

    /// The rules for a next legal move are as follows:
    /// 1. calculate any legal king moves first since we
    /// need to know this in check mate conditions anyway.
    /// 2. If actively in check, calculate any moves that
    /// can block check.
    /// 3. If no moves came from 1 or 2, there are no
    /// additional legal moves. Check mate condition.
    /// 4. Calculate which pieces are currently blocking
    /// an opposing piece from creating a check. These
    /// are then used to rule out moves in each piece
    /// kind's calculations.
    /// 5. for each of the remaining piece kinds (pawn,
    /// knight, bishop, rook, queen) that are not actively
    /// defending an opposing check, calculate their legal
    /// moves.
    fn next(&mut self) -> Option<Self::Item> {
        if !self.king_moves_iterator_finished {
            let next_king_move = self.king_moves_iterator.next();
            if next_king_move.is_some() {
                return next_king_move;
            } else {
                self.king_moves_iterator_finished = true;
            }
        }

        // lazy calculate whether we are in check.
        let is_check = match self.is_check {
            None => {
                let is_check = self
                    .king_moves_iterator
                    .is_check(self.player, self.board.kings[self.player.as_index()].0);
                self.is_check = Some(is_check);
                is_check
            }
            Some(is_check) => is_check,
        };

        if let Some(pawn_moves) = &mut self.pawn_moves_iterator {
            let next_pawn_move = pawn_moves.next();
            if next_pawn_move.is_some() {
                return next_pawn_move;
            } else {
                self.pawn_moves_iterator = None;
            }
        }

        /*if let Some(knight_moves) = &mut self.knight_moves_iterator {
            let next_knight_move = knight_moves.next();
            if next_knight_move.is_some() {
                return next_knight_move;
            } else {
                self.knight_moves_iterator = None;
            }
        }

        if let Some(bishop_moves) = &mut self.bishop_moves_iterator {
            let next_bishop_move = bishop_moves.next();
            if next_bishop_move.is_some() {
                return next_bishop_move;
            } else {
                self.bishop_moves_iterator = None;
            }
        }

        if let Some(rook_moves) = &mut self.rook_moves_iterator {
            let next_rook_move = rook_moves.next();
            if next_rook_move.is_some() {
                return next_rook_move;
            } else {
                self.rook_moves_iterator = None;
            }
        }

        if let Some(queen_moves) = &mut self.queen_moves_iterator {
            let next_queen_move = queen_moves.next();
            if next_queen_move.is_some() {
                return next_queen_move;
            } else {
                self.queen_moves_iterator = None;
            }
        }

        return None;*/
        todo!();
    }
}

struct LegalCapturesAtLocationIterator<'board> {
    board: &'board Board,
    player_to_move: usize,
    target_square: u64,
    attacking_defending_pieces_mailbox: Option<(u64, u64)>,
    knight_shifts_to_check: Option<IntoIter<usize, 4>>,
    knight_right_shift_on_deck: Option<usize>,
    other_shifts_to_check: Option<IntoIter<usize, 4>>,
    other_right_shift_on_deck: Option<usize>,
}

impl<'board> LegalCapturesAtLocationIterator<'board> {
    fn new(board: &'board Board, player_to_move: Player, target: u64) -> Self {
        debug_assert!(
            Location::try_from(target).is_ok(),
            "{} is an invalid location u64",
            target
        );

        Self {
            board,
            player_to_move: player_to_move.as_index(),
            target_square: target,
            attacking_defending_pieces_mailbox: None,
            knight_shifts_to_check: Some(shifts::knights().into_iter()),
            knight_right_shift_on_deck: None,
            other_shifts_to_check: Some(shifts::all().into_iter()),
            other_right_shift_on_deck: None,
        }
    }

    fn handle_knight_shift_on_deck(&mut self) -> Option<Move> {
        if let Some(knight_shift) = std::mem::take(&mut self.knight_right_shift_on_deck) {
            let knight_square = self.target_square >> knight_shift;
            if knight_square & self.board.knights[self.player_to_move].0 != 0 {
                return Some(Move {
                    from: Location::try_from(knight_square)
                        .expect(Location::failed_from_usize_message()),
                    to: Location::try_from(self.target_square)
                        .expect(Location::failed_from_usize_message()),
                });
            }
        }

        return None;
    }

    fn handle_other_shift_on_deck(
        &mut self,
        attacking_pieces_mailbox: u64,
        defending_pieces_mailbox: u64,
    ) -> Option<Move> {
        if let Some(shift) = std::mem::take(&mut self.other_right_shift_on_deck) {
            let mut current_right = self.target_square >> shift;
            let mut is_first_right = true;
            loop {
                if defending_pieces_mailbox & current_right != 0 {
                    return None;
                }

                if attacking_pieces_mailbox & current_right == 0 {
                    is_first_right = false;
                    continue;
                }

                if shifts::is_diagonal(shift) {
                    if self.board.bishops[self.player_to_move].0 & current_right != 0 {
                        return Some(Move {
                            from: Location::try_from(current_right)
                                .expect(Location::failed_from_usize_message()),
                            to: Location::try_from(self.target_square)
                                .expect(Location::failed_from_usize_message()),
                        });
                    }

                    if self.board.queens[self.player_to_move].0 & current_right != 0 {
                        return Some(Move {
                            from: Location::try_from(current_right)
                                .expect(Location::failed_from_usize_message()),
                            to: Location::try_from(self.target_square)
                                .expect(Location::failed_from_usize_message()),
                        });
                    }

                    // pawns and kings can only attack 1 square away
                    if !is_first_right {
                        continue;
                    }

                    if self.board.kings[self.player_to_move].0 & current_right != 0 {
                        return Some(Move {
                            from: Location::try_from(current_right)
                                .expect(Location::failed_from_usize_message()),
                            to: Location::try_from(self.target_square)
                                .expect(Location::failed_from_usize_message()),
                        });
                    }

                    // Since this is a right shift, even if pawns are in these
                    // positions, they wouldn't be attacking the king.
                    if let black!() = self.player_to_move {
                        continue;
                    }

                    if (shift == ((up_left!() as i32).abs() as usize)
                        || shift == ((up_right!() as i32).abs() as usize))
                        && self.board.pawns[black!()].0 & current_right != 0
                    {
                        return Some(Move {
                            from: Location::try_from(current_right)
                                .expect(Location::failed_from_usize_message()),
                            to: Location::try_from(self.target_square)
                                .expect(Location::failed_from_usize_message()),
                        });
                    }
                } else if shifts::is_straight(shift) {
                    if self.board.rooks[self.player_to_move].0 & current_right != 0 {
                        return Some(Move {
                            from: Location::try_from(current_right)
                                .expect(Location::failed_from_usize_message()),
                            to: Location::try_from(self.target_square)
                                .expect(Location::failed_from_usize_message()),
                        });
                    }

                    if self.board.queens[self.player_to_move].0 & current_right != 0 {
                        return Some(Move {
                            from: Location::try_from(current_right)
                                .expect(Location::failed_from_usize_message()),
                            to: Location::try_from(self.target_square)
                                .expect(Location::failed_from_usize_message()),
                        });
                    }

                    if !is_first_right {
                        continue;
                    }

                    if self.board.kings[self.player_to_move].0 & current_right != 0 {
                        return Some(Move {
                            from: Location::try_from(current_right)
                                .expect(Location::failed_from_usize_message()),
                            to: Location::try_from(self.target_square)
                                .expect(Location::failed_from_usize_message()),
                        });
                    }
                }

                is_first_right = false;
                current_right = current_right >> shift;
            }
        }

        return None;
    }
}

impl<'board> Iterator for LegalCapturesAtLocationIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.board.assert_board_integrity();

        // lazy calculate our mailboxes of pieces. These help
        // speed up some checks later on.
        let defending_pieces_mailbox;
        let attacking_pieces_mailbox;
        if let Some((attacking_mailbox, defending_mailbox)) =
            self.attacking_defending_pieces_mailbox
        {
            attacking_pieces_mailbox = attacking_mailbox;
            defending_pieces_mailbox = defending_mailbox;
        } else {
            let white_pieces_mailbox = self.board.pawns[white!()].0
                | self.board.knights[white!()].0
                | self.board.bishops[white!()].0
                | self.board.rooks[white!()].0
                | self.board.queens[white!()].0;
            // Omit the kings. We only care about the opponent's king

            let black_pieces_mailbox = self.board.pawns[black!()].0
                | self.board.knights[black!()].0
                | self.board.bishops[black!()].0
                | self.board.rooks[black!()].0
                | self.board.queens[black!()].0;
            // Omit the kings. We only care about the opponent's king

            match self.player_to_move {
                white!() => {
                    defending_pieces_mailbox = white_pieces_mailbox;
                    attacking_pieces_mailbox = black_pieces_mailbox | self.board.kings[black!()].0;
                }
                black!() => {
                    defending_pieces_mailbox = black_pieces_mailbox;
                    attacking_pieces_mailbox = white_pieces_mailbox | self.board.kings[white!()].0;
                }
                value => unreachable!("{}", value),
            }
            self.attacking_defending_pieces_mailbox =
                Some((attacking_pieces_mailbox, defending_pieces_mailbox));
        }
        let white_pieces_mailbox = self.board.pawns[white!()].0
            | self.board.knights[white!()].0
            | self.board.bishops[white!()].0
            | self.board.rooks[white!()].0
            | self.board.queens[white!()].0;
        // Omit the kings. We only care about the opponent's king

        let black_pieces_mailbox = self.board.pawns[black!()].0
            | self.board.knights[black!()].0
            | self.board.bishops[black!()].0
            | self.board.rooks[black!()].0
            | self.board.queens[black!()].0;
        // Omit the kings. We only care about the opponent's king

        let defending_pieces_mailbox;
        let attacking_pieces_mailbox;
        match self.player_to_move {
            white!() => {
                defending_pieces_mailbox = white_pieces_mailbox;
                attacking_pieces_mailbox = black_pieces_mailbox | self.board.kings[black!()].0;
            }
            black!() => {
                defending_pieces_mailbox = black_pieces_mailbox;
                attacking_pieces_mailbox = white_pieces_mailbox | self.board.kings[white!()].0;
            }
            value => unreachable!("{}", value),
        }

        if let Some(move_) = self.handle_knight_shift_on_deck() {
            return Some(move_);
        }

        if let Some(mut knight_shifts_to_check) = std::mem::take(&mut self.knight_shifts_to_check) {
            while let Some(knight_shift) = knight_shifts_to_check.next() {
                self.knight_right_shift_on_deck = Some(knight_shift);

                let new_location = self.target_square << knight_shift;
                if new_location & self.board.knights[self.player_to_move].0 != 0 {
                    self.knight_shifts_to_check = Some(knight_shifts_to_check);
                    return Some(Move {
                        from: Location::try_from(new_location)
                            .expect(Location::failed_from_usize_message()),
                        to: Location::try_from(new_location)
                            .expect(Location::failed_from_usize_message()),
                    });
                }

                if let Some(move_) = self.handle_knight_shift_on_deck() {
                    self.knight_shifts_to_check = Some(knight_shifts_to_check);
                    return Some(move_);
                }
            }
        }

        if let Some(move_) =
            self.handle_other_shift_on_deck(attacking_pieces_mailbox, defending_pieces_mailbox)
        {
            return Some(move_);
        }

        if let Some(mut other_shifts) = std::mem::take(&mut self.other_shifts_to_check) {
            while let Some(shift) = other_shifts.next() {
                self.other_right_shift_on_deck = Some(shift);

                let mut current_left = self.target_square << shift;
                let mut is_first_left = true;
                loop {
                    if defending_pieces_mailbox & current_left != 0 {
                        break;
                    }

                    if attacking_pieces_mailbox & current_left == 0 {
                        is_first_left = false;
                        continue;
                    }

                    if shifts::is_diagonal(shift) {
                        if self.board.bishops[self.player_to_move].0 & current_left != 0 {
                            self.other_shifts_to_check = Some(other_shifts);
                            return Some(Move {
                                from: Location::try_from(current_left)
                                    .expect(Location::failed_from_usize_message()),
                                to: Location::try_from(self.target_square)
                                    .expect(Location::failed_from_usize_message()),
                            });
                        }

                        if self.board.queens[self.player_to_move].0 & current_left != 0 {
                            self.other_shifts_to_check = Some(other_shifts);
                            return Some(Move {
                                from: Location::try_from(current_left)
                                    .expect(Location::failed_from_usize_message()),
                                to: Location::try_from(self.target_square)
                                    .expect(Location::failed_from_usize_message()),
                            });
                        }

                        // pawns and kings can only attack 1 square away
                        if !is_first_left {
                            continue;
                        }

                        if self.board.kings[self.player_to_move].0 & current_left != 0 {
                            self.other_shifts_to_check = Some(other_shifts);
                            return Some(Move {
                                from: Location::try_from(current_left)
                                    .expect(Location::failed_from_usize_message()),
                                to: Location::try_from(self.target_square)
                                    .expect(Location::failed_from_usize_message()),
                            });
                        }

                        // Since this is a left shift, even if pawns are in these
                        // positions, they wouldn't be attacking the king.
                        if let white!() = self.player_to_move {
                            continue;
                        }

                        if (shift == ((down_right!() as i32).abs() as usize)
                            || shift == ((down_left!() as i32).abs() as usize))
                            && self.board.pawns[black!()].0 & current_left != 0
                        {
                            self.other_shifts_to_check = Some(other_shifts);
                            return Some(Move {
                                from: Location::try_from(current_left)
                                    .expect(Location::failed_from_usize_message()),
                                to: Location::try_from(self.target_square)
                                    .expect(Location::failed_from_usize_message()),
                            });
                        }
                    }

                    if shifts::is_straight(shift) {
                        if self.board.rooks[self.player_to_move].0 & current_left != 0 {
                            self.other_shifts_to_check = Some(other_shifts);
                            return Some(Move {
                                from: Location::try_from(current_left)
                                    .expect(Location::failed_from_usize_message()),
                                to: Location::try_from(self.target_square)
                                    .expect(Location::failed_from_usize_message()),
                            });
                        }

                        if self.board.queens[self.player_to_move].0 & current_left != 0 {
                            self.other_shifts_to_check = Some(other_shifts);
                            return Some(Move {
                                from: Location::try_from(current_left)
                                    .expect(Location::failed_from_usize_message()),
                                to: Location::try_from(self.target_square)
                                    .expect(Location::failed_from_usize_message()),
                            });
                        }

                        if !is_first_left {
                            continue;
                        }

                        if self.board.kings[self.player_to_move].0 & current_left != 0 {
                            self.other_shifts_to_check = Some(other_shifts);
                            return Some(Move {
                                from: Location::try_from(current_left)
                                    .expect(Location::failed_from_usize_message()),
                                to: Location::try_from(self.target_square)
                                    .expect(Location::failed_from_usize_message()),
                            });
                        }
                    }

                    is_first_left = false;
                    current_left = current_left << shift;
                }

                if let Some(move_) = self
                    .handle_other_shift_on_deck(attacking_pieces_mailbox, defending_pieces_mailbox)
                {
                    return Some(move_);
                }
            }
        }

        return None;
    }
}

struct CheckData {
    squares_to_block_active_check: Vec<[Option<u64>; 8]>,
    squares_that_are_currently_blocking_check: [Option<u64>; 8],
}

struct LegalKingMovesIterator<'board> {
    board: &'board Board,
    player: Player,
    king_bitboard: Option<u64>,
    on_deck: Option<u64>,
    shifts: IntoIter<usize, 4>,
}

impl<'board> LegalKingMovesIterator<'board> {
    fn new(board: &'board Board, player: Player) -> Self {
        Self {
            board,
            player,
            king_bitboard: None,
            on_deck: None,
            shifts: shifts::all().into_iter(),
        }
    }

    fn is_check(&self, player: Player, king_position: u64) -> bool {
        return LegalCapturesAtLocationIterator::new(&self.board, player, king_position)
            .next()
            .is_some();
    }
}

impl<'board> Iterator for LegalKingMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let king_bitboard = match self.king_bitboard {
            None => {
                let king_bitboard = match self.player {
                    Player::White => self.board.kings[white!()].0,
                    Player::Black => self.board.kings[black!()].0,
                };
                self.king_bitboard = Some(king_bitboard);
                king_bitboard
            }
            Some(king_bitboard) => king_bitboard,
        };

        if let Some(on_deck) = std::mem::take(&mut self.on_deck) {
            let new_location = king_bitboard >> on_deck;

            if !self.is_check(self.player, new_location) {
                return Some(Move {
                    from: Location::try_from(king_bitboard).expect("Only one king to exist"),
                    to: Location::try_from(new_location).expect("Only one new location to exist"),
                });
            }
        }

        while let Some(next_shift) = self.shifts.next() {
            let left = king_bitboard << next_shift;
            let right = king_bitboard >> next_shift;

            if !self.is_check(self.player, left) {
                self.on_deck = Some(right);
                return Some(Move {
                    from: Location::try_from(king_bitboard).expect("Only one king to exist"),
                    to: Location::try_from(left).expect("Only one new location to exist"),
                });
            }

            if !self.is_check(self.player, right) {
                return Some(Move {
                    from: Location::try_from(king_bitboard).expect("Only one king to exist"),
                    to: Location::try_from(right).expect("Only one new location to exists"),
                });
            }
        }

        None
    }
}

struct LegalPawnMovesIterator<'board> {
    board: &'board Board,
    checked_en_passants: bool,
    en_passant_lookahead: Option<Move>,
}

impl<'board> LegalPawnMovesIterator<'board> {
    fn new(board: &'board Board) -> Self {
        Self {
            board,
            checked_en_passants: false,
            en_passant_lookahead: None,
        }
    }
}

impl<'board> Iterator for LegalPawnMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.checked_en_passants {
            self.checked_en_passants = true;
            if let Some(last_move) = self.board.history.last() {
                let moved_piece = self.board.at(last_move.to).expect("Board integrity issue: last recorded move describes a move to a square that now has no piece.");

                if (last_move.to.rank().as_int() - last_move.from.rank().as_int()).abs() == 2 {
                    if let PieceKind::Pawn = moved_piece.kind {
                        let player_to_move = self.board.get_player_to_move().as_index();

                        let move_bits = last_move.to.as_u64();
                        let player_to_move_pawns = self.board.pawns[player_to_move].0;

                        let en_passant_left = move_bits << shifts::right!();
                        let en_passant_right = move_bits >> shifts::right!();

                        if en_passant_right & player_to_move_pawns != 0 {
                            self.en_passant_lookahead = Some(Move {
                                from: Location::try_from(en_passant_right)
                                    .expect(Location::failed_from_usize_message()),
                                to: Location::try_from(match player_to_move {
                                    white!() => move_bits << shifts::down!(),
                                    black!() => move_bits >> shifts::down!(),
                                    value => unreachable!("{value}"),
                                })
                                .expect(Location::failed_from_usize_message()),
                            });
                        }

                        if en_passant_left & player_to_move_pawns != 0 {
                            return Some(Move {
                                from: Location::try_from(en_passant_left)
                                    .expect(Location::failed_from_usize_message()),
                                to: Location::try_from(match player_to_move {
                                    white!() => move_bits << shifts::down!(),
                                    black!() => move_bits >> shifts::down!(),
                                    value => unreachable!("{value}"),
                                })
                                .expect(Location::failed_from_usize_message()),
                            });
                        }

                        if let Some(lookahead) = std::mem::take(&mut self.en_passant_lookahead) {
                            return Some(lookahead);
                        }
                    }
                }
            }
        }

        todo!("standard");
    }
}

struct LegalKnightMovesIterator<'board> {
    board: &'board Board,
}

struct LegalBishopMovesIterator<'board> {
    board: &'board Board,
}

struct LegalRookMovesIterator<'board> {
    board: &'board Board,
}

struct LegalQueenMovesIterator<'board> {
    board: &'board Board,
}

enum LegalMovesGeneratorState {
    None,
    PawnGeneration,
    KnightGeneration,
    BishopGeneration,
    RookGeneration,
    QueenGeneration,
    KingGeneration,
}

impl From<Piece> for LegalMovesGeneratorState {
    fn from(value: Piece) -> Self {
        Self::from(value.kind)
    }
}

impl From<PieceKind> for LegalMovesGeneratorState {
    fn from(value: PieceKind) -> Self {
        match value {
            PieceKind::Pawn => LegalMovesGeneratorState::PawnGeneration,
            PieceKind::Knight => LegalMovesGeneratorState::KnightGeneration,
            PieceKind::Bishop => LegalMovesGeneratorState::BishopGeneration,
            PieceKind::Rook => LegalMovesGeneratorState::RookGeneration,
            PieceKind::Queen => LegalMovesGeneratorState::QueenGeneration,
            PieceKind::King => LegalMovesGeneratorState::KingGeneration,
        }
    }
}

mod shifts {
    macro_rules! left {
        () => {
            -1
        };
    }
    pub(super) use left;

    macro_rules! right {
        () => {
            1
        };
    }
    pub(super) use right;

    macro_rules! up {
        () => {
            -8
        };
    }
    pub(super) use up;

    macro_rules! down {
        () => {
            8
        };
    }
    pub(super) use down;

    macro_rules! up_left {
        () => {
            -9
        };
    }
    pub(super) use up_left;

    macro_rules! up_right {
        () => {
            -7
        };
    }
    pub(super) use up_right;

    macro_rules! down_left {
        () => {
            7
        };
    }
    pub(super) use down_left;

    macro_rules! down_right {
        () => {
            9
        };
    }
    pub(super) use down_right;

    pub(super) fn all() -> [usize; 4] {
        [down_left!(), down!(), down_right!(), right!()]
    }

    pub(super) fn straights() -> [usize; 2] {
        [down!(), right!()]
    }

    pub(super) fn diagnoals() -> [usize; 2] {
        [down_left!(), down_right!()]
    }

    pub(super) fn knights() -> [usize; 4] {
        [6, 10, 15, 17]
    }

    pub(super) fn is_diagonal(shift: usize) -> bool {
        match shift {
            down_left!() | down_right!() => true,
            _ => false,
        }
    }

    pub(super) fn is_straight(shift: usize) -> bool {
        match shift {
            down!() | right!() => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use chess_common::File;

    use crate::chess_move::Move;

    use super::Board;

    #[test]
    fn classifies_pawn_moves_correctly() {
        let starting_state = Board::default();
    }
}
