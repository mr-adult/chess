use chess_common::{Location, Player, Rank};

use crate::{bitboard::BitBoard, Board, Move};
use arr_deque::ArrDeque;

pub(crate) struct LegalPawnMovesIterator<'board> {
    board: &'board Board,
    hostiles: BitBoard,
    lookahead: ArrDeque<Move, 5>,
    pawn_locations: Box<dyn Iterator<Item = Location>>,
}

impl<'board> LegalPawnMovesIterator<'board> {
    pub(crate) fn new(board: &'board Board) -> Self {
        let moving_player = board.player_to_move();
        let hostile_player = moving_player.other_player();
        Self {
            board,
            hostiles: board.create_mailbox_for_player(hostile_player),
            lookahead: ArrDeque::new(),
            pawn_locations: Box::new(Location::from_bitboard(
                board.pawns[moving_player.as_index()].0,
            )),
        }
    }
}

impl<'board> Iterator for LegalPawnMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(lookahead) = self.lookahead.pop_front() {
            return Some(lookahead);
        }

        while let Some(location) = self.pawn_locations.next() {
            let location_bb = BitBoard::new(location.as_u64());
            match self.board.player_to_move() {
                Player::White => {
                    let new_location = location_bb.up();

                    // check for a double-push
                    if location.rank() == Rank::Two {
                        let new_location_double = new_location.up();
                        if new_location_double.0 != 0
                            // Can't double-push through another piece
                            && !new_location.intersects_with(&self.board.mailbox)
                            && !new_location_double.intersects_with(&self.board.mailbox)
                        {
                            let result = self.lookahead.push_back(Move {
                                from: location.clone(),
                                to: Location::try_from(new_location_double.0)
                                    .expect(Location::failed_from_usize_message()),
                            });
                            debug_assert!(result.is_ok());
                        }
                    }

                    // check for captures
                    let en_passant_target = self
                        .board
                        .en_passant_target_square()
                        .map(|loc| loc.as_u64())
                        .unwrap_or(0u64);

                    for capture_square in [location_bb.up_left(), location_bb.up_right()] {
                        if capture_square.0 != 0
                            && (capture_square.intersects_with(&self.hostiles)
                                || capture_square.intersects_with_u64(en_passant_target))
                        {
                            let result = self.lookahead.push_back(Move {
                                from: location.clone(),
                                to: Location::try_from(capture_square.0).expect(
                                    "Conversion of capture square to location should never fail",
                                ),
                            });
                            debug_assert!(result.is_ok());
                        }
                    }

                    if new_location.0 != 0 && !new_location.intersects_with(&self.board.mailbox) {
                        return Some(Move {
                            from: location,
                            to: Location::try_from(new_location.0).unwrap(),
                        });
                    }

                    if let Some(lookahead) = self.lookahead.pop_front() {
                        return Some(lookahead);
                    }
                }
                Player::Black => {
                    let new_location = location_bb.down();

                    if location.rank() == Rank::Seven {
                        let new_location_double = new_location.down();
                        if new_location_double.0 != 0
                            // Can't double-push through another piece.
                            && !new_location.intersects_with(&self.board.mailbox)
                            && !new_location_double.intersects_with(&self.board.mailbox)
                        {
                            let result = self.lookahead.push_back(Move {
                                from: location.clone(),
                                to: Location::try_from(new_location_double.0)
                                    .expect(Location::failed_from_usize_message()),
                            });
                            debug_assert!(result.is_ok());
                        }
                    }

                    // check for captures
                    let en_passant_target = self
                        .board
                        .en_passant_target_square()
                        .map(|loc| loc.as_u64())
                        .unwrap_or(0u64);

                    for capture_square in [location_bb.down_left(), location_bb.down_right()] {
                        if capture_square.0 != 0
                            && (capture_square.intersects_with(&self.hostiles)
                                || capture_square.intersects_with_u64(en_passant_target))
                        {
                            let result = self.lookahead.push_back(Move {
                                from: location.clone(),
                                to: Location::try_from(capture_square.0).expect(
                                    "Conversion of capture square to location should never fail",
                                ),
                            });
                            debug_assert!(result.is_ok());
                        }
                    }

                    if new_location.0 != 0 && !new_location.intersects_with(&self.board.mailbox) {
                        return Some(Move {
                            from: location,
                            to: Location::try_from(new_location.0)
                                .expect(Location::failed_from_usize_message()),
                        });
                    }

                    if let Some(lookahead) = self.lookahead.pop_front() {
                        return Some(lookahead);
                    }
                }
            }
        }

        return None;
    }
}
