use std::{
    array::IntoIter,
    simd::{num::SimdUint, u64x2, u64x4},
    str::FromStr,
};

use arr_deque::ArrDeque;
use chess_common::{File, Location, Player, Rank};

use crate::{bitboard::BitBoard, Board, Move};

pub(crate) struct LegalKingMovesIterator<'board> {
    board: &'board Board,
    player: Player,
    king_bitboard: BitBoard,
    moves: IntoIter<u64x4, 2>,
    queued: ArrDeque<u64, 4>,
    checked_castle_queenside: bool,
    checked_castle_kingside: bool,
}

impl<'board> LegalKingMovesIterator<'board> {
    pub(crate) fn new(board: &'board Board, player: Player) -> Self {
        let player_index = player.as_index();
        let king_bitboard = board.kings[player_index].clone();

        let friendly_pieces = board.create_mailbox_for_player(player);
        let friendly_pieces_mask = u64x4::splat(!friendly_pieces.0);

        let king_moves = [
            u64x4::from_array([
                BitBoard::up_u64(king_bitboard.0),
                BitBoard::left_u64(king_bitboard.0),
                BitBoard::down_u64(king_bitboard.0),
                BitBoard::right_u64(king_bitboard.0),
            ]) & friendly_pieces_mask,
            u64x4::from_array([
                BitBoard::up_right_u64(king_bitboard.0),
                BitBoard::up_left_u64(king_bitboard.0),
                BitBoard::down_right_u64(king_bitboard.0),
                BitBoard::down_left_u64(king_bitboard.0),
            ]) & friendly_pieces_mask,
        ];

        Self {
            board,
            player,
            moves: king_moves.into_iter(),
            queued: ArrDeque::new(),
            king_bitboard,
            checked_castle_kingside: false,
            checked_castle_queenside: false,
        }
    }

    pub(crate) fn simd_is_check(board: &Board, player: Player, king_positions: u64x4) -> u64x4 {
        let player_index = player.as_index();
        let other_player = player.other_player();
        let other_player_index = other_player.as_index();

        let friendlies = board.pawns[player_index].0
            | board.knights[player_index].0
            | board.bishops[player_index].0
            | board.rooks[player_index].0
            | board.queens[player_index].0;

        let enemy_king_pawns_knights = board.kings[other_player_index].0
            | board.pawns[other_player_index].0
            | board.knights[other_player_index].0;

        let bitwise_not_friendlies = !friendlies;

        let candidate_pawns = match other_player {
            Player::White => {
                BitBoard::simd_down_left(king_positions) | BitBoard::simd_down_right(king_positions)
            }
            Player::Black => {
                BitBoard::simd_up_left(king_positions) | BitBoard::simd_up_right(king_positions)
            }
        };

        let pawns_result = u64x4::splat(board.pawns[other_player_index].0) & candidate_pawns;

        let verticals = BitBoard::simd_up(BitBoard::simd_up(king_positions))
            | BitBoard::simd_down(BitBoard::simd_down(king_positions));

        let horizontals = BitBoard::simd_left(BitBoard::simd_left(king_positions))
            | BitBoard::simd_right(BitBoard::simd_right(king_positions));

        let knight_mask = BitBoard::simd_left(verticals)
            | BitBoard::simd_right(verticals)
            | BitBoard::simd_up(horizontals)
            | BitBoard::simd_down(horizontals);

        // Check for knight captures.
        let knights_result = knight_mask & u64x4::splat(board.knights[other_player_index].0);

        let left_shifters = [
            BitBoard::UP_LEFT_DIR_LEFT_SHIFT_OFFSET as u64,
            BitBoard::UP_RIGHT_DIR_LEFT_SHIFT_OFFSET as u64,
        ]
        .map(u64x4::splat);
        let right_shifters = [
            BitBoard::DOWN_LEFT_DIR_RIGHT_SHIFT_OFFSET as u64,
            BitBoard::DOWN_RIGHT_DIR_RIGHT_SHIFT_OFFSET as u64,
        ]
        .map(u64x4::splat);
        let bitwise_not_friendlies_splat = u64x2::splat(bitwise_not_friendlies);
        let enemy_non_bishop_likes =
            !(enemy_king_pawns_knights | board.rooks[other_player_index].0);
        let left_shift_masks = [
            !File::h_bit_filter()
                & !Rank::one_bit_filter()
                & bitwise_not_friendlies
                & enemy_non_bishop_likes,
            !File::a_bit_filter()
                & !Rank::one_bit_filter()
                & bitwise_not_friendlies
                & enemy_non_bishop_likes,
        ]
        .map(u64x4::splat);
        let right_shift_masks = [
            !File::h_bit_filter()
                & !Rank::eight_bit_filter()
                & bitwise_not_friendlies
                & enemy_non_bishop_likes,
            !File::a_bit_filter()
                & !Rank::eight_bit_filter()
                & bitwise_not_friendlies
                & enemy_non_bishop_likes,
        ]
        .map(u64x4::splat);

        let mut left_shift_aggregator_1 = king_positions.clone();
        let mut left_shift_aggregator_2 = king_positions.clone();
        let mut right_shift_aggregator_1 = king_positions.clone();
        let mut right_shift_aggregator_2 = king_positions.clone();
        for _ in 0..7 {
            left_shift_aggregator_1 = left_shift_aggregator_1
                | ((left_shift_aggregator_1 << left_shifters[0]) & left_shift_masks[0]);
            left_shift_aggregator_2 = left_shift_aggregator_2
                | ((left_shift_aggregator_2 << left_shifters[1]) & left_shift_masks[1]);
            right_shift_aggregator_1 = right_shift_aggregator_1
                | ((right_shift_aggregator_1 >> right_shifters[0]) & right_shift_masks[0]);
            right_shift_aggregator_2 = right_shift_aggregator_2
                | ((right_shift_aggregator_2 >> right_shifters[1]) & right_shift_masks[1]);
        }

        let bishop_mask = (left_shift_aggregator_1
            | left_shift_aggregator_2
            | right_shift_aggregator_1
            | right_shift_aggregator_2)
            & !(king_positions.clone());

        let enemy_bishop_likes =
            u64x4::splat(board.bishops[other_player_index].0 | board.queens[other_player_index].0);
        let bishop_result = bishop_mask & enemy_bishop_likes;

        let left_shifters = u64x2::from_array([
            BitBoard::RIGHT_DIR_LEFT_SHIFT_OFFSET as u64,
            BitBoard::UP_DIR_LEFT_SHIFT_OFFSET as u64,
        ]);
        let right_shifters = u64x2::from_array([
            BitBoard::LEFT_DIR_RIGHT_SHIFT_OFFSET as u64,
            BitBoard::DOWN_DIR_RIGHT_SHIFT_OFFSET as u64,
        ]);
        let enemy_non_rook_likes =
            u64x2::splat(!(enemy_king_pawns_knights | board.bishops[other_player_index].0));
        let left_mask = (u64x2::from_array([!File::a_bit_filter(), !Rank::one_bit_filter()])
            & bitwise_not_friendlies_splat
            & enemy_non_rook_likes)
            .to_array()
            .map(u64x4::splat);
        let right_mask = (u64x2::from_array([!File::h_bit_filter(), !Rank::eight_bit_filter()])
            & bitwise_not_friendlies_splat
            & enemy_non_rook_likes)
            .to_array()
            .map(u64x4::splat);

        let mut left_shift_aggregator_1 = king_positions.clone();
        let mut left_shift_aggregator_2 = king_positions.clone();
        let mut right_shift_aggregator_1 = king_positions.clone();
        let mut right_shift_aggregator_2 = king_positions.clone();
        for _ in 0..7 {
            left_shift_aggregator_1 = left_shift_aggregator_1
                | ((left_shift_aggregator_1 << left_shifters[0]) & left_mask[0]);
            left_shift_aggregator_2 = left_shift_aggregator_2
                | ((left_shift_aggregator_2 << left_shifters[1]) & left_mask[1]);
            right_shift_aggregator_1 = right_shift_aggregator_1
                | ((right_shift_aggregator_1 >> right_shifters[0]) & right_mask[0]);
            right_shift_aggregator_2 = right_shift_aggregator_2
                | ((right_shift_aggregator_2 >> right_shifters[1]) & right_mask[1]);
        }

        let rook_mask = (left_shift_aggregator_1
            | left_shift_aggregator_2
            | right_shift_aggregator_1
            | right_shift_aggregator_2)
            & !(king_positions.clone());

        let enemy_rook_likes =
            board.rooks[other_player_index].0 | board.queens[other_player_index].0;
        let rook_result = rook_mask & u64x4::splat(enemy_rook_likes);

        let king_mask = BitBoard::simd_up(king_positions)
            | BitBoard::simd_left(king_positions)
            | BitBoard::simd_down(king_positions)
            | BitBoard::simd_right(king_positions)
            | BitBoard::simd_up_left(king_positions)
            | BitBoard::simd_up_right(king_positions)
            | BitBoard::simd_down_left(king_positions)
            | BitBoard::simd_down_right(king_positions);
        let king_result = king_mask & u64x4::splat(board.kings[other_player_index].0);

        return pawns_result | knights_result | bishop_result | rook_result | king_result;
    }

    pub(crate) fn is_check(board: &Board, player: Player, king_position: u64) -> bool {
        let player_index = player.as_index();
        let other_player = player.other_player();
        let other_player_index = other_player.as_index();

        let friendlies = board.pawns[player_index].0
            | board.knights[player_index].0
            | board.bishops[player_index].0
            | board.rooks[player_index].0
            | board.queens[player_index].0;

        let enemy_king_pawns_knights = board.kings[other_player_index].0
            | board.pawns[other_player_index].0
            | board.knights[other_player_index].0;

        let bitwise_not_friendlies = !friendlies;

        let candidate_pawns = match other_player {
            Player::White => {
                BitBoard::down_left_u64(king_position) | BitBoard::down_right_u64(king_position)
            }
            Player::Black => {
                BitBoard::up_left_u64(king_position) | BitBoard::up_right_u64(king_position)
            }
        };

        if (board.pawns[other_player_index].0 & candidate_pawns) != 0 {
            return true;
        }

        let verticals = u64x2::from_array([
            BitBoard::up_u64(BitBoard::up_u64(king_position)),
            BitBoard::down_u64(BitBoard::down_u64(king_position)),
        ]);
        let horizontals = u64x2::from_array([
            BitBoard::left_u64(BitBoard::left_u64(king_position)),
            BitBoard::right_u64(BitBoard::right_u64(king_position)),
        ]);

        let knight_mask = (BitBoard::simd_left(verticals)
            | BitBoard::simd_right(verticals)
            | BitBoard::simd_up(horizontals)
            | BitBoard::simd_down(horizontals))
        .reduce_or();

        // Check for knight captures.
        if (knight_mask & board.knights[other_player_index].0) != 0 {
            return true;
        }

        let left_shifters = u64x2::from_array([
            BitBoard::UP_LEFT_DIR_LEFT_SHIFT_OFFSET as u64,
            BitBoard::UP_RIGHT_DIR_LEFT_SHIFT_OFFSET as u64,
        ]);
        let right_shifters = u64x2::from_array([
            BitBoard::DOWN_LEFT_DIR_RIGHT_SHIFT_OFFSET as u64,
            BitBoard::DOWN_RIGHT_DIR_RIGHT_SHIFT_OFFSET as u64,
        ]);
        let mut left_shift_masks = u64x2::from_array([
            !File::h_bit_filter() & !Rank::one_bit_filter(),
            !File::a_bit_filter() & !Rank::one_bit_filter(),
        ]);
        let mut right_shift_masks = u64x2::from_array([
            !File::h_bit_filter() & !Rank::eight_bit_filter(),
            !File::a_bit_filter() & !Rank::eight_bit_filter(),
        ]);
        let bitwise_not_friendlies_splat = u64x2::splat(bitwise_not_friendlies);
        let enemy_non_bishop_likes =
            u64x2::splat(!(enemy_king_pawns_knights | board.rooks[other_player_index].0));
        left_shift_masks = left_shift_masks & bitwise_not_friendlies_splat & enemy_non_bishop_likes;
        right_shift_masks =
            right_shift_masks & bitwise_not_friendlies_splat & enemy_non_bishop_likes;

        let king_splat_2 = u64x2::splat(king_position);
        let mut left_shift_aggregator = king_splat_2.clone();
        let mut right_shift_aggregator = king_splat_2.clone();
        for _ in 0..7 {
            left_shift_aggregator = left_shift_aggregator
                | ((left_shift_aggregator << left_shifters) & left_shift_masks);
            right_shift_aggregator = right_shift_aggregator
                | ((right_shift_aggregator >> right_shifters) & right_shift_masks);
        }

        let bishop_mask =
            (left_shift_aggregator | right_shift_aggregator).reduce_or() & !king_position;
        let enemy_bishop_likes =
            board.bishops[other_player_index].0 | board.queens[other_player_index].0;
        if (bishop_mask & enemy_bishop_likes) != 0 {
            return true;
        }

        let left_shifters = u64x2::from_array([
            BitBoard::RIGHT_DIR_LEFT_SHIFT_OFFSET as u64,
            BitBoard::UP_DIR_LEFT_SHIFT_OFFSET as u64,
        ]);
        let right_shifters = u64x2::from_array([
            BitBoard::LEFT_DIR_RIGHT_SHIFT_OFFSET as u64,
            BitBoard::DOWN_DIR_RIGHT_SHIFT_OFFSET as u64,
        ]);
        let enemy_non_rook_likes =
            u64x2::splat(!(enemy_king_pawns_knights | board.bishops[other_player_index].0));
        let left_mask = u64x2::from_array([!File::a_bit_filter(), !Rank::one_bit_filter()])
            & bitwise_not_friendlies_splat
            & enemy_non_rook_likes;
        let right_mask = u64x2::from_array([!File::h_bit_filter(), !Rank::eight_bit_filter()])
            & bitwise_not_friendlies_splat
            & enemy_non_rook_likes;

        let mut left_shift_aggregator = king_splat_2.clone();
        let mut right_shift_aggregator = king_splat_2.clone();
        for _ in 0..7 {
            left_shift_aggregator =
                left_shift_aggregator | ((left_shift_aggregator << left_shifters) & left_mask);
            right_shift_aggregator =
                right_shift_aggregator | ((right_shift_aggregator >> right_shifters) & right_mask);
        }

        let rook_mask =
            (left_shift_aggregator | right_shift_aggregator).reduce_or() & !king_position;
        let enemy_rook_likes =
            board.rooks[other_player_index].0 | board.queens[other_player_index].0;
        if (rook_mask & enemy_rook_likes) != 0 {
            return true;
        }

        let king_mask = BitBoard::up_u64(king_position)
            | BitBoard::left_u64(king_position)
            | BitBoard::down_u64(king_position)
            | BitBoard::right_u64(king_position)
            | BitBoard::up_left_u64(king_position)
            | BitBoard::up_right_u64(king_position)
            | BitBoard::down_left_u64(king_position)
            | BitBoard::down_right_u64(king_position);
        (king_mask & board.kings[other_player_index].0) != 0
    }
}

impl<'board> Iterator for LegalKingMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.board.assert_board_integrity();

        let king_bitboard = &self.king_bitboard;

        if let Some(queued) = self.queued.pop_front() {
            return Some(Move {
                from: Location::expect_from(king_bitboard.0),
                to: Location::expect_from(queued),
            });
        }

        while let Some(king_move_set) = self.moves.next() {
            if king_move_set.reduce_or() == 0 {
                continue;
            }

            let king_moves = king_move_set.to_array();
            let is_check =
                LegalKingMovesIterator::simd_is_check(self.board, self.player, king_move_set)
                    .to_array();
            for i in 0..king_moves.len() {
                if king_moves[i] == 0 {
                    continue;
                }

                // Non-zero denotes it was a check.
                if is_check[i] != 0 {
                    continue;
                }

                self.queued
                    .push_back(king_moves[i])
                    .expect("Failed to push to the queue.");
            }

            if let Some(queued) = self.queued.pop_front() {
                return Some(Move {
                    from: Location::expect_from(king_bitboard.0),
                    to: Location::expect_from(queued),
                });
            }
        }

        if self.checked_castle_queenside && self.checked_castle_kingside {
            return None;
        }

        if !self.checked_castle_queenside && !self.checked_castle_kingside {
            if LegalKingMovesIterator::is_check(self.board, self.player, self.king_bitboard.0) {
                return None;
            }
        }

        let castle_rank = match self.player {
            Player::White => Rank::One,
            Player::Black => Rank::Eight,
        };

        if !self.checked_castle_queenside {
            self.checked_castle_queenside = true;

            if self.board.player_can_castle_queenside(&self.player) {
                let any_pieces_in_way = || {
                    [File::b, File::c, File::d]
                        .into_iter()
                        .map(|file| Location::new(file, castle_rank))
                        .map(|loc| loc.as_u64())
                        .any(|bitboard| self.board.mailbox.intersects_with_u64(bitboard))
                };

                let any_checks_in_way = || {
                    [File::c, File::d]
                        .into_iter()
                        .map(|file| Location::new(file, castle_rank))
                        .any(|loc| Self::is_check(&self.board, self.player, loc.as_u64()))
                };

                let to_loc = Location::new(File::c, castle_rank);
                if !any_pieces_in_way()
                    && !any_checks_in_way()
                    && !Self::is_check(self.board, self.player, to_loc.as_u64())
                {
                    return Some(Move {
                        from: Location::expect_from(self.king_bitboard.0),
                        to: to_loc,
                    });
                }
            }
        }

        if !self.checked_castle_kingside {
            self.checked_castle_kingside = true;

            if self.board.player_can_castle_kingside(&self.player) {
                let any_pieces_in_way = [File::f, File::g]
                    .into_iter()
                    .map(|file| Location::new(file, castle_rank))
                    .map(|loc| loc.as_u64())
                    .any(|bitboard| self.board.mailbox.intersects_with_u64(bitboard));

                let any_checks_in_way = [File::f, File::g]
                    .into_iter()
                    .map(|file| Location::new(file, castle_rank))
                    .any(|loc| Self::is_check(&self.board, self.player, loc.as_u64()));

                let to_loc = Location::new(File::g, castle_rank);

                if !any_pieces_in_way
                    && !any_checks_in_way
                    && !Self::is_check(self.board, self.player, to_loc.as_u64())
                {
                    return Some(Move {
                        from: Location::expect_from(self.king_bitboard.0),
                        to: to_loc,
                    });
                }
            }
        }

        return None;
    }
}

// Tests should live in the legal_moves module
