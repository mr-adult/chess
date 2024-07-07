use std::vec;

use chess_common::Location;

use crate::{bitboard::BitBoard, Board, Move};

use super::all_iterator::RookMovesIterator;

pub(crate) struct LegalRookMovesIterator<'board> {
    #[allow(unused)]
    board: &'board Board,
    rook_locations: vec::IntoIter<Location>,
    current_rook_data: Option<CurrentRookData>,
    friendlies: BitBoard,
    hostiles: BitBoard,
}

struct CurrentRookData {
    from_location: Location,
    to_locations: RookMovesIterator,
}

impl<'board> LegalRookMovesIterator<'board> {
    pub(crate) fn new(board: &'board Board) -> Self {
        let player_to_move = board.player_to_move();
        let other_player = player_to_move.other_player();

        let friendlies = board.create_mailbox_for_player(player_to_move);
        let hostiles = board.create_mailbox_for_player(other_player);

        Self {
            board: board,
            rook_locations: Location::from_bitboard(
                board.rooks[board.player_to_move().as_index()].0,
            ),
            current_rook_data: None,
            friendlies,
            hostiles,
        }
    }

    pub(crate) fn new_for_bitboard(board: &'board Board, bitboard: &BitBoard) -> Self {
        let mut result = Self::new(board);
        result.rook_locations = Location::from_bitboard(bitboard.0);
        result
    }
}

impl<'board> Iterator for LegalRookMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current_rook_data = match &mut self.current_rook_data {
                None => match self.rook_locations.next() {
                    None => return None,
                    Some(location) => {
                        let location_u64 = location.as_u64();
                        let move_data = CurrentRookData {
                            from_location: location,
                            to_locations: RookMovesIterator::new(BitBoard::new(location_u64)),
                        };
                        self.current_rook_data = Some(move_data);
                        self.current_rook_data.as_mut().unwrap()
                    }
                },
                Some(move_data) => move_data,
            };

            match current_rook_data.to_locations.next() {
                None => {
                    self.current_rook_data = None;
                }
                Some(to_location) => {
                    if self.friendlies.intersects_with(&to_location) {
                        current_rook_data.to_locations.next_direction();
                        continue;
                    }

                    if self.hostiles.intersects_with(&to_location) {
                        current_rook_data.to_locations.next_direction();
                    }

                    return Some(Move {
                        from: current_rook_data.from_location.clone(),
                        to: Location::try_from(to_location.0)
                            .expect(Location::failed_from_usize_message()),
                    });
                }
            }
        }
    }
}
