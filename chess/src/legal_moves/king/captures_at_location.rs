use std::array::IntoIter;

use chess_common::{Location, Player};

use crate::{
    bitboard::BitBoard,
    legal_moves::{
        bishop::BishopMovesIterator, knight::KnightMovesIterator, rook::RookMovesIterator,
    },
    Board, Move,
};

#[derive(Debug)]
pub(super) struct LegalCapturesAtLocationIterator<'board> {
    board: &'board Board,
    player_to_move: usize,
    target_square_location: Location,
    pawn_moves: IntoIter<BitBoard, 2>,
    pawn_moves_is_done: bool,
    knight_moves: KnightMovesIterator,
    knight_moves_is_done: bool,
    diagonal_moves: BishopMovesIterator,
    diagonal_moves_is_done: bool,
    straight_moves: RookMovesIterator,
    straight_moves_is_done: bool,
    king_moves: IntoIter<BitBoard, 8>,
    king_moves_is_done: bool,
    mailbox: BitBoard,
}

impl<'board> LegalCapturesAtLocationIterator<'board> {
    pub(super) fn new(board: &'board Board, player_to_move: Player, target: u64) -> Self {
        debug_assert!(
            Location::try_from(target).is_ok(),
            "{} is an invalid location u64",
            target
        );

        let target_bb = BitBoard::new(target);
        let king_moves = [
            target_bb.up(),
            target_bb.up_right(),
            target_bb.right(),
            target_bb.down_right(),
            target_bb.down(),
            target_bb.down_left(),
            target_bb.left(),
            target_bb.up_left(),
        ]
        .into_iter();

        let target_square_location =
            Location::try_from(target).expect(Location::failed_from_usize_message());

        Self {
            board,
            player_to_move: player_to_move.as_index(),
            target_square_location,
            pawn_moves: match player_to_move {
                Player::White => [target_bb.down_left(), target_bb.down_right()].into_iter(),
                Player::Black => [target_bb.up_left(), target_bb.up_right()].into_iter(),
            },
            pawn_moves_is_done: false,
            knight_moves: KnightMovesIterator::new(target_bb.clone()),
            knight_moves_is_done: false,
            diagonal_moves: BishopMovesIterator::new(target_bb.clone()),
            diagonal_moves_is_done: false,
            straight_moves: RookMovesIterator::new(target_bb),
            straight_moves_is_done: false,
            king_moves,
            king_moves_is_done: false,
            mailbox: board.mailbox.clone(),
        }
    }

    pub(super) fn new_with_mailbox(
        board: &'board Board,
        player_to_move: Player,
        target: u64,
        mailbox: u64,
    ) -> Self {
        let mut result = Self::new(board, player_to_move, target);
        result.mailbox = BitBoard::new(mailbox);
        result
    }
}

impl<'board> Iterator for LegalCapturesAtLocationIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.board.assert_board_integrity();

        if !self.pawn_moves_is_done {
            while let Some(attacking_pawn_square) = self.pawn_moves.next() {
                if self.board.pawns[self.player_to_move].intersects_with(&attacking_pawn_square) {
                    return Some(Move {
                        from: Location::try_from(attacking_pawn_square.0)
                            .expect(Location::failed_from_usize_message()),
                        to: self.target_square_location.clone(),
                    });
                }
            }

            self.pawn_moves_is_done = true;
        }

        if !self.knight_moves_is_done {
            while let Some(attacking_knight_square) = self.knight_moves.next() {
                if self.board.knights[self.player_to_move].intersects_with(&attacking_knight_square)
                {
                    return Some(Move {
                        from: Location::try_from(attacking_knight_square.0)
                            .expect(Location::failed_from_usize_message()),
                        to: self.target_square_location.clone(),
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
                        to: self.target_square_location.clone(),
                    });
                }

                if self.mailbox.intersects_with(&diagonal_move)
                    && !self.diagonal_moves.next_direction()
                {
                    break;
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
                        to: self.target_square_location.clone(),
                    });
                }

                if self.mailbox.intersects_with(&straight_move)
                    && !self.straight_moves.next_direction()
                {
                    break;
                }
            }

            self.straight_moves_is_done = true;
        }

        if !self.king_moves_is_done {
            while let Some(king_move) = self.king_moves.next() {
                if self.board.kings[self.player_to_move].intersects_with(&king_move) {
                    return Some(Move {
                        from: Location::try_from(king_move.0)
                            .expect(Location::failed_from_usize_message()),
                        to: self.target_square_location.clone(),
                    });
                }
            }

            self.king_moves_is_done = true;
        }

        return None;
    }
}
