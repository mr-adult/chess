use std::{
    array::{from_fn, IntoIter},
    iter,
    ops::{Index, IndexMut},
    str::FromStr,
};

use chess_common::{black, white, File, Location, Piece, PieceKind, Player, Rank};
use chess_parsers::{parse_fen, BoardLayout, FenErr};

use crate::{
    arr_deque::ArrDeque,
    bitboard::{BitBoard, DiagonalMovesIterator, KnightMovesIterator, StraightMovesIterator},
    chess_move::{IllegalMove, MoveKind, PawnMoveKind},
    Move,
};

#[derive(Debug)]
pub struct ErgonomicBoard {
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
pub struct Board {
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
        let starting_player = self.starting_position.player_to_move();
        if self.history.len() % 2 == 0 {
            result.push(starting_player.as_char());
        } else {
            result.push(starting_player.other_player().as_char());
        }

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

    fn create_mailbox_for_player(&self, player: Player) -> BitBoard {
        let player_index = player.as_index();
        return BitBoard(
            self.pawns[player_index].0
                | self.knights[player_index].0
                | self.bishops[player_index].0
                | self.rooks[player_index].0
                | self.queens[player_index].0
                | self.kings[player_index].0,
        );
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

    fn en_passant_target_square(&self) -> Option<Location> {
        if let Some(last_move) = self.history.last() {
            if let Some(last_moved) = self.at(last_move.to) {
                if let PieceKind::Pawn = last_moved.kind() {
                    // It may not be necessary to check the files, but it is safer.
                    if last_move.to.file() == last_move.from.file()
                        && last_move.to.rank().as_int() - last_move.from.rank().as_int() == 2
                    {
                        let file = last_move.to.file();
                        let rank = last_move.from.rank().as_int() + last_move.to.rank().as_int()
                            - last_move.from.rank().as_int();
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
}

impl Default for Board {
    fn default() -> Self {
        Self::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

struct LegalMovesIterator<'board> {
    board: &'board Board,
    player: Player,
    is_check: Option<bool>,
    pawn_moves_iterator: Option<LegalPawnMovesIterator<'board>>,
    knight_moves_iterator: Option<LegalKnightMovesIterator>,
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
            knight_moves_iterator: Some(LegalKnightMovesIterator::new(board)),
            bishop_moves_iterator: Some(LegalBishopMovesIterator::new(board)),
            rook_moves_iterator: Some(LegalRookMovesIterator::new(board)),
            queen_moves_iterator: Some(LegalQueenMovesIterator::new(board)),
            king_moves_iterator: LegalKingMovesIterator::new(board, player_to_move),
            king_moves_iterator_finished: false,
            check_blocking_squares: None,
        }
    }

    fn for_piece(board: &'board Board, piece: Piece) -> Self {
        Self {
            board,
            player: piece.player(),
            is_check: None,
            pawn_moves_iterator: Some(LegalPawnMovesIterator::new(board)),
            knight_moves_iterator: Some(LegalKnightMovesIterator::new(board)),
            bishop_moves_iterator: Some(LegalBishopMovesIterator::new(board)),
            rook_moves_iterator: Some(LegalRookMovesIterator::new(board)),
            queen_moves_iterator: Some(LegalQueenMovesIterator::new(board)),
            king_moves_iterator: LegalKingMovesIterator::new(&board, piece.player()),
            // We only want to iterate through king moves if the target piece is a king
            king_moves_iterator_finished: piece.kind() != PieceKind::King,
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
        // let is_check = self.king_moves_iterator.is_check(self.player, self.board.kings[self.player.as_index()].0);
        // println!("Finished calculating is_check: {is_check}");

        if let Some(pawn_moves) = &mut self.pawn_moves_iterator {
            let next_pawn_move = pawn_moves.next();
            if next_pawn_move.is_some() {
                return next_pawn_move;
            } else {
                self.pawn_moves_iterator = None;
            }
        }

        if let Some(knight_moves) = &mut self.knight_moves_iterator {
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

        return None;
    }
}

#[derive(Debug)]
struct LegalCapturesAtLocationIterator<'board> {
    board: &'board Board,
    player_to_move: usize,
    target_square: u64,
    attacking_defending_pieces_mailbox: Option<(u64, u64)>,
    knight_moves: KnightMovesIterator,
    knight_moves_is_done: bool,
    diagonal_moves: DiagonalMovesIterator,
    diagonal_moves_is_done: bool,
    straight_moves: StraightMovesIterator,
    straight_moves_is_done: bool,
}

impl<'board> LegalCapturesAtLocationIterator<'board> {
    fn new(board: &'board Board, player_to_move: Player, target: u64) -> Self {
        debug_assert!(
            Location::try_from(target).is_ok(),
            "{} is an invalid location u64",
            target
        );

        let target_bb = BitBoard(target);

        Self {
            board,
            player_to_move: player_to_move.as_index(),
            target_square: target,
            attacking_defending_pieces_mailbox: None,
            knight_moves: target_bb.knight_moves(),
            knight_moves_is_done: false,
            diagonal_moves: target_bb.diagonal_moves(),
            diagonal_moves_is_done: false,
            straight_moves: target_bb.straight_moves(),
            straight_moves_is_done: false,
        }
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

        if !self.knight_moves_is_done {
            while let Some(attacking_knight_square) = self.knight_moves.next() {
                if self.board.knights[self.player_to_move].intersects_with_u64(self.target_square) {
                    return Some(Move {
                        from: Location::try_from(attacking_knight_square.0)
                            .expect(Location::failed_from_usize_message()),
                        to: Location::try_from(self.target_square)
                            .expect(Location::failed_from_usize_message()),
                    });
                }
            }
            self.knight_moves_is_done = true;
        }

        if !self.diagonal_moves_is_done {
            while let Some(diagonal_move) = self.diagonal_moves.next() {
                if self.board.bishops[self.player_to_move].intersects_with(&diagonal_move)
                    || self.board.queens[self.player_to_move].intersects_with(&diagonal_move)
                {
                    return Some(Move {
                        from: Location::try_from(diagonal_move.0)
                            .expect(Location::failed_from_usize_message()),
                        to: Location::try_from(self.target_square)
                            .expect(Location::failed_from_usize_message()),
                    });
                }
            }

            self.diagonal_moves_is_done = true;
        }

        if !self.straight_moves_is_done {
            while let Some(straight_move) = self.straight_moves.next() {
                if self.board.rooks[self.player_to_move].intersects_with(&straight_move)
                    || self.board.queens[self.player_to_move].intersects_with(&straight_move)
                {
                    return Some(Move {
                        from: Location::try_from(straight_move.0)
                            .expect(Location::failed_from_usize_message()),
                        to: Location::try_from(self.target_square)
                            .expect(Location::failed_from_usize_message()),
                    });
                }
            }

            self.straight_moves_is_done = true;
        }

        return None;
    }
}

struct LegalKingMovesIterator<'board> {
    board: &'board Board,
    player: Player,
    king_bitboard: BitBoard,
    moves: IntoIter<BitBoard, 8>,
    friendly_pieces: u64,
}

impl<'board> LegalKingMovesIterator<'board> {
    fn new(board: &'board Board, player: Player) -> Self {
        let player_index = player.as_index();
        let king_bitboard = board.kings[player_index].clone();
        Self {
            board,
            player,
            king_bitboard: king_bitboard.clone(),
            moves: [
                king_bitboard.up(),
                king_bitboard.up_right(),
                king_bitboard.right(),
                king_bitboard.down_right(),
                king_bitboard.down(),
                king_bitboard.down_left(),
                king_bitboard.left(),
                king_bitboard.up_left(),
            ]
            .into_iter(),
            friendly_pieces: board.pawns[player_index].0
                | board.knights[player_index].0
                | board.bishops[player_index].0
                | board.rooks[player_index].0
                | board.queens[player_index].0,
        }
    }

    fn is_check(&self, player: Player, king_position: u64) -> bool {
        let mut iterator = LegalCapturesAtLocationIterator::new(&self.board, player, king_position);
        iterator.next().is_some()
    }
}

impl<'board> Iterator for LegalKingMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let king_bitboard = self.king_bitboard.clone();
        let friendlies = self.friendly_pieces;

        while let Some(king_move) = self.moves.next() {
            if king_move.0 == 0 {
                continue;
            }

            if king_move.intersects_with_u64(friendlies) {
                continue;
            }

            if self.is_check(self.player, self.king_bitboard.0) {
                continue;
            }

            return Some(Move {
                from: Location::try_from(king_bitboard.0)
                    .expect(Location::failed_from_usize_message()),
                to: Location::try_from(king_move.0).expect(Location::failed_from_usize_message()),
            });
        }

        None
    }
}

struct LegalPawnMovesIterator<'board> {
    board: &'board Board,
    checked_en_passants: bool,
    hostiles: BitBoard,
    lookahead: ArrDeque<Move, 3>,
    locations: Box<dyn Iterator<Item = Location>>,
}

impl<'board> LegalPawnMovesIterator<'board> {
    fn new(board: &'board Board) -> Self {
        let hostile_player = board.get_player_to_move().other_player().as_index();
        Self {
            board,
            checked_en_passants: false,
            hostiles: BitBoard(
                board.pawns[hostile_player].0
                    | board.knights[hostile_player].0
                    | board.bishops[hostile_player].0
                    | board.rooks[hostile_player].0
                    | board.queens[hostile_player].0
                    | board.kings[hostile_player].0,
            ),
            lookahead: ArrDeque::new(),
            locations: Box::new(Location::all_locations()),
        }
    }
}

impl<'board> Iterator for LegalPawnMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(lookahead) = self.lookahead.pop_front() {
            return Some(lookahead);
        }

        if !self.checked_en_passants {
            self.checked_en_passants = true;
            if let Some(last_move) = self.board.history.last() {
                let moved_piece = self.board.at(last_move.to).expect("Board integrity issue: last recorded move describes a move to a square that now has no piece.");

                if (last_move.to.rank().as_int() - last_move.from.rank().as_int()).abs() == 2 {
                    if let PieceKind::Pawn = moved_piece.kind() {
                        let player_to_move = self.board.get_player_to_move().as_index();

                        let move_bb = BitBoard(last_move.to.as_u64());
                        let player_to_move_pawns = self.board.pawns[player_to_move].clone();

                        let en_passant_left = move_bb.left();
                        let en_passant_right = move_bb.right();

                        if en_passant_left.intersects_with(&player_to_move_pawns) {
                            debug_assert!(self
                                .lookahead
                                .push_back(Move {
                                    from: Location::try_from(en_passant_left.0)
                                        .expect(Location::failed_from_usize_message()),
                                    to: Location::try_from(match player_to_move {
                                        white!() => en_passant_left.up_right().0,
                                        black!() => en_passant_left.down_right().0,
                                        _ => unreachable!(),
                                    })
                                    .expect(Location::failed_from_usize_message()),
                                })
                                .is_ok());
                        }

                        if en_passant_right.intersects_with(&player_to_move_pawns) {
                            debug_assert!(self
                                .lookahead
                                .push_back(Move {
                                    from: Location::try_from(en_passant_right.0)
                                        .expect(Location::failed_from_usize_message()),
                                    to: Location::try_from(match player_to_move {
                                        white!() => en_passant_left.up_left().0,
                                        black!() => en_passant_right.down_left().0,
                                        _ => unreachable!(),
                                    })
                                    .expect(Location::failed_from_usize_message())
                                })
                                .is_ok());
                        }

                        if let Some(lookahead) = self.lookahead.pop_front() {
                            return Some(lookahead);
                        }
                    }
                }
            }
        }

        while let Some(location) = self.locations.next() {
            let location_bb = BitBoard(location.as_u64());
            if self.board.pawns[self.board.get_player_to_move().as_index()]
                .intersects_with(&location_bb)
            {
                match self.board.get_player_to_move() {
                    Player::White => {
                        let new_location = location_bb.up();

                        // check for a double-push
                        if location.rank() == Rank::Two {
                            let new_location_double = new_location.up();
                            if new_location_double.0 != 0
                                && !new_location_double.intersects_with(&self.board.mailbox)
                            {
                                debug_assert!(self
                                    .lookahead
                                    .push_back(Move {
                                        from: location,
                                        to: Location::try_from(new_location_double.0)
                                            .expect(Location::failed_from_usize_message()),
                                    })
                                    .is_ok());
                            }
                        }

                        // check for captures
                        for capture_square in [location_bb.up_left(), location_bb.up_right()] {
                            if capture_square.0 != 0
                                && capture_square.intersects_with(&self.hostiles)
                            {
                                debug_assert!(self.lookahead.push_back(Move {
                                    from: location,
                                    to: Location::try_from(capture_square.0).expect("Conversion of capture square to location should never fail"),
                                }).is_ok());
                            }
                        }

                        if new_location.0 != 0 && !new_location.intersects_with(&self.board.mailbox)
                        {
                            return Some(Move {
                                from: location,
                                to: Location::try_from(new_location.0).unwrap(),
                            });
                        }
                    }
                    Player::Black => {
                        let new_location = location_bb.down();

                        if location.rank() == Rank::Seven {
                            let new_location_double = new_location.down();
                            if new_location_double.0 != 0
                                && !new_location_double.intersects_with(&self.board.mailbox)
                            {
                                debug_assert!(self
                                    .lookahead
                                    .push_back(Move {
                                        from: location,
                                        to: Location::try_from(new_location_double.0)
                                            .expect(Location::failed_from_usize_message()),
                                    })
                                    .is_ok());
                            }
                        }

                        // check for captures
                        for capture_square in [location_bb.down_left(), location_bb.down_right()] {
                            if capture_square.0 != 0
                                && capture_square.intersects_with(&self.hostiles)
                            {
                                debug_assert!(self.lookahead.push_back(Move {
                                    from: location,
                                    to: Location::try_from(capture_square.0).expect("Conversion of capture square to location should never fail"),
                                }).is_ok());
                            }
                        }

                        if new_location.0 != 0 && !new_location.intersects_with(&self.board.mailbox)
                        {
                            return Some(Move {
                                from: location,
                                to: Location::try_from(new_location.0)
                                    .expect(Location::failed_from_usize_message()),
                            });
                        }
                    }
                }
            }
        }

        return None;
    }
}

struct LegalKnightMovesIterator {
    friendlies: BitBoard,
    knights: BitBoard,
    locations: Box<dyn Iterator<Item = Location>>,
    lookahead: ArrDeque<Move, 8>,
}

impl LegalKnightMovesIterator {
    fn new(board: &Board) -> Self {
        let player_to_move = board.get_player_to_move();
        Self {
            friendlies: board.create_mailbox_for_player(player_to_move),
            knights: board.knights[player_to_move.as_index()].clone(),
            locations: Box::new(Location::all_locations()),
            lookahead: ArrDeque::new(),
        }
    }
}

impl Iterator for LegalKnightMovesIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(front) = self.lookahead.pop_front() {
            return Some(front);
        }

        while let Some(location) = self.locations.next() {
            let location_bb = BitBoard(location.as_u64());
            if self.knights.intersects_with(&location_bb) {
                let mut iter = location_bb.knight_moves();
                while let Some(knight_move) = iter.next() {
                    if self.friendlies.intersects_with(&knight_move) {
                        continue;
                    }
                    debug_assert!(self
                        .lookahead
                        .push_back(Move {
                            from: location,
                            to: Location::try_from(knight_move.0)
                                .expect(Location::failed_from_usize_message()),
                        })
                        .is_ok());
                }

                if let Some(lookahead) = self.lookahead.pop_front() {
                    return Some(lookahead);
                }
            }
        }

        return None;
    }
}

struct LegalBishopMovesIterator<'board> {
    board: &'board Board,
}

impl<'board> LegalBishopMovesIterator<'board> {
    fn new(board: &'board Board) -> Self {
        Self { board }
    }
}

impl<'board> Iterator for LegalBishopMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        return None;
        todo!();
    }
}

struct LegalRookMovesIterator<'board> {
    board: &'board Board,
}

impl<'board> LegalRookMovesIterator<'board> {
    fn new(board: &'board Board) -> Self {
        Self { board }
    }
}

impl<'board> Iterator for LegalRookMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        return None;
        todo!();
    }
}

struct LegalQueenMovesIterator<'board> {
    bishop_moves: LegalBishopMovesIterator<'board>,
    bishop_moves_finished: bool,
    rook_moves: LegalRookMovesIterator<'board>,
    rook_moves_finished: bool,
}

impl<'board> LegalQueenMovesIterator<'board> {
    fn new(board: &'board Board) -> Self {
        Self {
            bishop_moves: LegalBishopMovesIterator::new(board),
            bishop_moves_finished: false,
            rook_moves: LegalRookMovesIterator::new(board),
            rook_moves_finished: false,
        }
    }
}

impl<'board> Iterator for LegalQueenMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.bishop_moves_finished {
            let next = self.bishop_moves.next();
            if next.is_some() {
                return next;
            } else {
                self.bishop_moves_finished = true;
            }
        }

        if !self.rook_moves_finished {
            let next = self.rook_moves.next();
            if next.is_some() {
                return next;
            } else {
                self.rook_moves_finished = true;
            }
        }

        return None;
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::board::LegalKingMovesIterator;

    use super::Board;

    #[test]
    fn gets_legal_king_moves() {
        let board =
            Board::from_str("RNBKQBNR/PPPPPPPP/8/8/8/8/pppppppp/rnbkqbnr w KQkq - 0 1").unwrap();

        let legal_king_moves =
            LegalKingMovesIterator::new(&board, board.get_player_to_move()).collect::<Vec<_>>();
        assert_eq!(0, legal_king_moves.len());
    }
}
