use arr_deque::ArrDeque;
use chess_common::{Location, Player};

use crate::{
    bitboard::BitBoard,
    legal_moves::{
        bishop::{BishopMovesIterator, DiagonalDirection},
        knight::KnightMovesIterator,
        rook::{RookMovesIterator, StraightDirection},
    },
    Board,
};

#[derive(Debug)]
pub(crate) struct CheckStoppingSquaresIterator<'board> {
    board: &'board Board,
    player_to_move: usize,
    target_square: u64,
    #[allow(unused)]
    #[cfg(debug_assertions)]
    target_square_location: Location,
    knight_moves: KnightMovesIterator,
    diagonal_moves: ArrDeque<BishopMovesIterator, 4>,
    straight_moves: ArrDeque<RookMovesIterator, 4>,
    result: ArrDeque<Location, 7>,
}

impl<'board> CheckStoppingSquaresIterator<'board> {
    pub(crate) fn new(board: &'board Board, player_to_move: Player, target: u64) -> Self {
        debug_assert!(
            Location::try_from(target).is_ok(),
            "{} is an invalid location u64",
            target
        );

        let target_bb = BitBoard::new(target);

        let bishop_moves_iters = DiagonalDirection::all()
            .into_iter()
            .map(|dir| BishopMovesIterator::with_directions([dir], target_bb.clone()))
            .collect::<ArrDeque<_, 4>>();

        let rook_moves_iters = StraightDirection::all()
            .into_iter()
            .map(|dir| RookMovesIterator::with_directions([dir], target_bb.clone()))
            .collect::<ArrDeque<_, 4>>();

        Self {
            board,
            player_to_move: player_to_move.as_index(),
            target_square: target,
            #[cfg(debug_assertions)]
            target_square_location: Location::try_from(target)
                .expect(Location::failed_from_usize_message()),
            knight_moves: KnightMovesIterator::new(target_bb.clone()),
            diagonal_moves: bishop_moves_iters,
            straight_moves: rook_moves_iters,
            result: ArrDeque::new(),
        }
    }

    fn resolve_into_result<const N: usize>(&mut self, values: ArrDeque<BitBoard, N>) -> bool {
        let locations = values
            .into_iter()
            .map(|bb| Location::try_from(bb.0).expect(Location::failed_from_usize_message()))
            .collect::<ArrDeque<_, 7>>();

        if self.result.is_empty() {
            self.result = locations;
            return true;
        } else {
            let mut old_results = ArrDeque::new();
            std::mem::swap(&mut old_results, &mut self.result);

            for result in old_results {
                if locations.iter().any(|location| *location == result) {
                    assert!(self.result.push_back(result).is_ok());
                }
            }

            return !self.result.is_empty();
        }
    }
}

impl<'board> Iterator for CheckStoppingSquaresIterator<'board> {
    type Item = Location;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.result.is_empty() {
            let front = self.result.pop_front();
            debug_assert!(front.is_some());
            return front;
        }

        self.board.assert_board_integrity();

        while let Some(attacking_knight_square) = self.knight_moves.next() {
            if self.board.knights[Player::other_player_usize(self.player_to_move)]
                .intersects_with_u64(self.target_square)
            {
                let mut resolution = ArrDeque::<_, 1>::new();
                assert!(resolution.push_back(attacking_knight_square).is_ok());
                if !self.resolve_into_result(resolution) {
                    return None;
                }
            }
        }

        while let Some(bishop_moves_iter) = self.diagonal_moves.pop_front() {
            let bishop_moves = bishop_moves_iter.collect::<ArrDeque<_, 7>>();
            let mut attacking_index = 0;
            let mut has_attacking_bishop = false;
            for (i, bishop_square) in bishop_moves.iter().enumerate() {
                if self.board.bishops[Player::other_player_usize(self.player_to_move)]
                    .intersects_with(bishop_square)
                    || self.board.queens[Player::other_player_usize(self.player_to_move)]
                        .intersects_with(bishop_square)
                {
                    attacking_index = i;
                    has_attacking_bishop = true;
                    break;
                }

                if self.board.mailbox.intersects_with(bishop_square) {
                    break;
                }
            }

            if has_attacking_bishop {
                if !self.resolve_into_result(
                    bishop_moves
                        .into_iter()
                        .take(attacking_index + 1)
                        .collect::<ArrDeque<_, 7>>(),
                ) {
                    return None;
                }
            }
        }

        while let Some(rook_moves_iter) = self.straight_moves.pop_front() {
            let rook_moves = rook_moves_iter.collect::<ArrDeque<_, 7>>();
            let mut attacking_index = 0;
            let mut has_attacking_rook = false;
            for (i, rook_square) in rook_moves.iter().enumerate() {
                if self.board.rooks[Player::other_player_usize(self.player_to_move)]
                    .intersects_with(rook_square)
                    || self.board.queens[Player::other_player_usize(self.player_to_move)]
                        .intersects_with(rook_square)
                {
                    attacking_index = i;
                    has_attacking_rook = true;
                }

                if self.board.mailbox.intersects_with(rook_square) {
                    break;
                }
            }

            if has_attacking_rook {
                if !self.resolve_into_result(
                    rook_moves
                        .into_iter()
                        .take(attacking_index + 1)
                        .collect::<ArrDeque<_, 7>>(),
                ) {
                    return None;
                }
            }
        }

        return self.result.pop_front();
    }
}
