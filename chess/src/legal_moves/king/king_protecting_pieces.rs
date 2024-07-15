use arr_deque::ArrDeque;
use chess_common::{Location, Player};

use crate::{
    bitboard::BitBoard,
    legal_moves::{
        bishop::{BishopMovesIterator, DiagonalDirection},
        rook::{RookMovesIterator, StraightDirection},
    },
    Board,
};

#[derive(Debug)]
pub(crate) struct KingProtectingLocationsIterator<'board> {
    board: &'board Board,
    player_to_move: usize,
    #[allow(unused)]
    #[cfg(debug_assertions)]
    target_square_location: Location,
    diagonal_moves: ArrDeque<BishopMovesIterator, 4>,
    straight_moves: ArrDeque<RookMovesIterator, 4>,
    friendly_mailbox: BitBoard,
    enemy_mailbox: BitBoard,
    result: ArrDeque<(Location, ArrDeque<Location, 7>), 8>,
}

impl<'board> KingProtectingLocationsIterator<'board> {
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
            #[cfg(debug_assertions)]
            target_square_location: Location::try_from(target)
                .expect(Location::failed_from_usize_message()),
            diagonal_moves: bishop_moves_iters,
            straight_moves: rook_moves_iters,
            friendly_mailbox: board.create_mailbox_for_player(player_to_move),
            enemy_mailbox: board.create_mailbox_for_player(player_to_move.other_player()),
            result: ArrDeque::new(),
        }
    }
}

impl<'board> Iterator for KingProtectingLocationsIterator<'board> {
    type Item = (Location, ArrDeque<Location, 7>);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.result.is_empty() {
            let front = self.result.pop_front();
            debug_assert!(front.is_some());
            return front;
        }

        self.board.assert_board_integrity();

        while let Some(bishop_moves_iter) = self.diagonal_moves.pop_front() {
            let mut friendly_square = None;
            let mut so_far = ArrDeque::<u64, 7>::new();
            for bishop_move in bishop_moves_iter {
                if bishop_move.intersects_with(&self.friendly_mailbox) {
                    // 2 friendlies means neither is protecting the king by themself
                    if friendly_square.is_some() {
                        break;
                    }
                    friendly_square = Some(bishop_move);
                    continue;
                }

                if self.board.bishops[Player::other_player_usize(self.player_to_move)]
                    .intersects_with(&bishop_move)
                    || self.board.queens[Player::other_player_usize(self.player_to_move)]
                        .intersects_with(&bishop_move)
                {
                    if let Some(friendly) = friendly_square {
                        let ok = so_far.push_back(bishop_move.0);
                        debug_assert!(ok.is_ok());
                        let result = self.result.push_back((
                            Location::try_from(friendly.0)
                                .expect(Location::failed_from_usize_message()),
                            so_far
                                .into_iter()
                                .map(|bb| {
                                    Location::try_from(bb)
                                        .expect(Location::failed_from_usize_message())
                                })
                                .collect(),
                        ));
                        debug_assert!(result.is_ok());
                    }
                    break;
                }

                if self.enemy_mailbox.intersects_with(&bishop_move) {
                    break;
                }

                let result = so_far.push_back(bishop_move.0);
                debug_assert!(result.is_ok());
            }
        }

        while let Some(rook_moves_iter) = self.straight_moves.pop_front() {
            let mut friendly_square = None;
            let mut so_far = ArrDeque::<u64, 7>::new();
            for rook_move in rook_moves_iter {
                if rook_move.intersects_with(&self.friendly_mailbox) {
                    // 2 friendlies means neither is protecting the king by themself
                    if friendly_square.is_some() {
                        break;
                    }
                    friendly_square = Some(rook_move);
                    continue;
                }

                if self.board.rooks[Player::other_player_usize(self.player_to_move)]
                    .intersects_with(&rook_move)
                    || self.board.queens[Player::other_player_usize(self.player_to_move)]
                        .intersects_with(&rook_move)
                {
                    if let Some(friendly) = friendly_square {
                        let ok = so_far.push_back(rook_move.0);
                        debug_assert!(ok.is_ok());
                        assert!(self
                            .result
                            .push_back((
                                Location::try_from(friendly.0)
                                    .expect(Location::failed_from_usize_message()),
                                so_far
                                    .into_iter()
                                    .map(|bb| Location::try_from(bb)
                                        .expect(Location::failed_from_usize_message()))
                                    .collect()
                            ))
                            .is_ok());
                        break;
                    }
                }

                if self.enemy_mailbox.intersects_with(&rook_move) {
                    break;
                }

                let result = so_far.push_back(rook_move.0);
                debug_assert!(result.is_ok());
            }
        }

        return self.result.pop_front();
    }
}
