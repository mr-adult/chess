use chess_common::Location;

use crate::{ bitboard::BitBoard, Board, Move};
use arr_deque::ArrDeque;

pub(crate) struct LegalKnightMovesIterator {
    friendlies: BitBoard,
    knights: BitBoard,
    locations: Box<dyn Iterator<Item = Location>>,
    lookahead: ArrDeque<Move, 8>,
}

impl LegalKnightMovesIterator {
    pub(crate) fn new(board: &Board) -> Self {
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