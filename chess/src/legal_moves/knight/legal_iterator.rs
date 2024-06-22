use chess_common::Location;

use crate::{ bitboard::BitBoard, Board, Move};
use arr_deque::ArrDeque;

use super::KnightMovesIterator;

pub(crate) struct LegalKnightMovesIterator<'board> {
    board: &'board Board,
    friendlies: BitBoard,
    locations: Box<dyn Iterator<Item = Location>>,
    lookahead: ArrDeque<Move, 8>,
}

impl<'board> LegalKnightMovesIterator<'board> {
    pub(crate) fn new(board: &'board Board) -> Self {
        let player_to_move = board.get_player_to_move();
        let player_to_move_index = player_to_move.as_index();
        Self {
            board: &board,
            friendlies: board.create_mailbox_for_player(player_to_move),
            locations: Box::new(Location::from_bitboard(
                board.knights[player_to_move_index].0,
            )),
            lookahead: ArrDeque::new(),
        }
    }
}

impl<'board> Iterator for LegalKnightMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(front) = self.lookahead.pop_front() {
            return Some(front);
        }

        while let Some(location) = self.locations.next() {
            let mut iter = KnightMovesIterator::new(BitBoard::new(location.as_u64()));
            while let Some(knight_move) = iter.next() {
                if self.friendlies.intersects_with(&knight_move) {
                    continue;
                }

                assert!(self
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

        return None;
    }
}
