use std::vec::IntoIter;

use chess_common::Location;

use crate::{bitboard::BitBoard, Board, Move};

use super::all_iterator::BishopMovesIterator;


pub(crate) struct LegalBishopMovesIterator<'board> {
    #[allow(unused)]
    board: &'board Board,
    bishop_locations: IntoIter<Location>,
    current_bishop_data: Option<CurrentBishopData>,
    friendlies: BitBoard,
    hostiles: BitBoard,
}

struct CurrentBishopData {
    from_location: Location,
    to_locations: BishopMovesIterator,
}

impl<'board> LegalBishopMovesIterator<'board> {
    pub(crate) fn new(board: &'board Board) -> Self {
        let player_to_move = board.get_player_to_move();
        let other_player = player_to_move.other_player();

        let friendlies = board.create_mailbox_for_player(player_to_move);
        let hostiles = board.create_mailbox_for_player(other_player);

        Self {
            board: board,
            bishop_locations: Location::from_bitboard(
                board.bishops[board.get_player_to_move().as_index()].0,
            ),
            current_bishop_data: None,
            friendlies,
            hostiles,
        }
    }

    pub(crate) fn new_for_bitboard(board: &'board Board, bitboard: &'board BitBoard) -> Self {
        let mut result = Self::new(board);
        result.bishop_locations = Location::from_bitboard(bitboard.0);
        result
    }
}

impl<'board> Iterator for LegalBishopMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current_bishop_data = match &mut self.current_bishop_data {
                None => match self.bishop_locations.next() {
                    None => return None,
                    Some(location) => {
                        let move_data = CurrentBishopData {
                            from_location: location,
                            to_locations: BishopMovesIterator::new(BitBoard(location.as_u64())),
                        };
                        self.current_bishop_data = Some(move_data);
                        self.current_bishop_data.as_mut().unwrap()
                    }
                },
                Some(move_data) => move_data,
            };

            match current_bishop_data.to_locations.next() {
                None => {
                    self.current_bishop_data = None;
                }
                Some(to_location) => {
                    if self.friendlies.intersects_with(&to_location) {
                        current_bishop_data.to_locations.next_direction();
                        continue;
                    }

                    if self.hostiles.intersects_with(&to_location) {
                        current_bishop_data.to_locations.next_direction();
                    }

                    return Some(Move {
                        from: current_bishop_data.from_location,
                        to: Location::try_from(to_location.0)
                            .expect(Location::failed_from_usize_message()),
                    });
                }
            }
        }
    }
}