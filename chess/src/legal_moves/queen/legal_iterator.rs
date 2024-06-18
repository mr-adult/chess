use crate::legal_moves::bishop::LegalBishopMovesIterator;
use crate::legal_moves::rook::LegalRookMovesIterator;
use crate::{Board, Move};

pub(crate) struct LegalQueenMovesIterator<'board> {
    bishop_moves: LegalBishopMovesIterator<'board>,
    bishop_moves_finished: bool,
    rook_moves: LegalRookMovesIterator<'board>,
    rook_moves_finished: bool,
}

impl<'board> LegalQueenMovesIterator<'board> {
    pub(crate) fn new(board: &'board Board) -> Self {
        let player_to_move = board.get_player_to_move().as_index();
        let queen_bb = &board.queens[player_to_move];
        Self {
            bishop_moves: LegalBishopMovesIterator::new_for_bitboard(&board, queen_bb),
            bishop_moves_finished: false,
            rook_moves: LegalRookMovesIterator::new_for_bitboard(&board, queen_bb),
            rook_moves_finished: false,
        }
    }
}

impl<'board> Iterator for LegalQueenMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.bishop_moves_finished {
            let next = self.bishop_moves.next();
            if next.is_some() {
                return next;
            } else {
                self.bishop_moves_finished = true;
            }
        }

        if !self.rook_moves_finished {
            let next = self.rook_moves.next();
            if next.is_some() {
                return next;
            } else {
                self.rook_moves_finished = true;
            }
        }

        return None;
    }
}
