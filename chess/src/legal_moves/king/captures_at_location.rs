use chess_common::{black, white, Location, Player};

use crate::{bitboard::BitBoard, legal_moves::{bishop::BishopMovesIterator, knight::KnightMovesIterator, rook::RookMovesIterator}, Board, Move};

#[derive(Debug)]
pub(super) struct LegalCapturesAtLocationIterator<'board> {
    board: &'board Board,
    player_to_move: usize,
    target_square: u64,
    attacking_defending_pieces_mailbox: Option<(u64, u64)>,
    knight_moves: KnightMovesIterator,
    knight_moves_is_done: bool,
    diagonal_moves: BishopMovesIterator,
    diagonal_moves_is_done: bool,
    straight_moves: RookMovesIterator,
    straight_moves_is_done: bool,
}

impl<'board> LegalCapturesAtLocationIterator<'board> {
    pub(super) fn new(board: &'board Board, player_to_move: Player, target: u64) -> Self {
        debug_assert!(
            Location::try_from(target).is_ok(),
            "{} is an invalid location u64",
            target
        );

        let target_bb = BitBoard::new(target);

        Self {
            board,
            player_to_move: player_to_move.as_index(),
            target_square: target,
            attacking_defending_pieces_mailbox: None,
            knight_moves: KnightMovesIterator::new(target_bb.clone()),
            knight_moves_is_done: false,
            diagonal_moves: BishopMovesIterator::new(target_bb.clone()),
            diagonal_moves_is_done: false,
            straight_moves: RookMovesIterator::new(target_bb),
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