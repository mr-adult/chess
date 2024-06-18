use chess_common::{black, white, Location, Player};

use crate::{Board, Move};
mod bishop;
mod king;
mod knight;
mod pawn;
pub(crate) mod rook;
mod queen;

pub(super) use bishop::BishopMovesIterator;
use bishop::LegalBishopMovesIterator;
use king::LegalKingMovesIterator;
pub(super) use knight::KnightMovesIterator;
use knight::LegalKnightMovesIterator;
use pawn::LegalPawnMovesIterator;
pub(super) use rook::RookMovesIterator;
use rook::LegalRookMovesIterator;
use queen::LegalQueenMovesIterator;


pub(crate) struct LegalMovesIterator<'board> {
    board: &'board Board,
    player: Player,
    is_check: Option<bool>,
    pawn_moves_iterator: Option<LegalPawnMovesIterator<'board>>,
    knight_moves_iterator: Option<LegalKnightMovesIterator>,
    bishop_moves_iterator: Option<LegalBishopMovesIterator<'board>>,
    rook_moves_iterator: Option<LegalRookMovesIterator<'board>>,
    queen_moves_iterator: Option<LegalQueenMovesIterator<'board>>,
    king_moves_iterator: LegalKingMovesIterator<'board>,
    king_moves_iterator_finished: bool,
    check_blocking_squares: Option<[Option<Location>; 8]>,
}

impl<'board> LegalMovesIterator<'board> {
    pub(crate) fn for_board(board: &'board Board) -> Self {
        let player_to_move = board.get_player_to_move();
        Self {
            board,
            player: player_to_move,
            is_check: None,
            pawn_moves_iterator: Some(LegalPawnMovesIterator::new(board)),
            knight_moves_iterator: Some(LegalKnightMovesIterator::new(board)),
            bishop_moves_iterator: Some(LegalBishopMovesIterator::new(&board)),
            rook_moves_iterator: Some(LegalRookMovesIterator::new(board)),
            queen_moves_iterator: Some(LegalQueenMovesIterator::new(board)),
            king_moves_iterator: LegalKingMovesIterator::new(board, player_to_move),
            king_moves_iterator_finished: false,
            check_blocking_squares: None,
        }
    }

    /// This function calculates all squares where if the piece on
    /// that square moves, it will result in a check. The rules for
    /// this are as follows:
    /// 1. Draw 8 vectors out from the king: one per cardinal direction
    /// 2. If the vector hits the edge of the board, no check opportunities
    /// 3. If the vector hits 2 friendly pieces first, no check opportunities
    /// 4. If the vector hits an enemy piece, it is already a check and we do
    /// not include it
    /// 5. If the vector hits a friendly piece then an enemy piece,
    /// moving the friendly piece will result in a check. These moves
    /// are included.
    fn calculate_check_blocking_squares(&mut self) -> () {
        let defending_piece_mailbox;
        let attacking_piece_mailbox;
        let attacking_player;
        match self.player {
            Player::White => {
                defending_piece_mailbox = self.board.create_mailbox_for_player(Player::White);
                attacking_piece_mailbox = self.board.create_mailbox_for_player(Player::Black);
                attacking_player = Player::Black.as_index();
            }
            Player::Black => {
                defending_piece_mailbox = self.board.create_mailbox_for_player(Player::Black);
                attacking_piece_mailbox = self.board.create_mailbox_for_player(Player::White);
                attacking_player = Player::White.as_index();
            }
        }

        let king_bitboard = match self.player {
            Player::White => self.board.kings[white!()].0,
            Player::Black => self.board.kings[black!()].0,
        };
    }
}

impl<'board> Iterator for LegalMovesIterator<'board> {
    type Item = Move;

    /// The rules for a next legal move are as follows:
    /// 1. calculate any legal king moves first since we
    /// need to know this in check mate conditions anyway.
    /// 2. If actively in check, calculate any moves that
    /// can block check.
    /// 3. If no moves came from 1 or 2, there are no
    /// additional legal moves. Check mate condition.
    /// 4. Calculate which pieces are currently blocking
    /// an opposing piece from creating a check. These
    /// are then used to rule out moves in each piece
    /// kind's calculations.
    /// 5. for each of the remaining piece kinds (pawn,
    /// knight, bishop, rook, queen) that are not actively
    /// defending an opposing check, calculate their legal
    /// moves.
    fn next(&mut self) -> Option<Self::Item> {
        if !self.king_moves_iterator_finished {
            let next_king_move = self.king_moves_iterator.next();
            if next_king_move.is_some() {
                return next_king_move;
            } else {
                self.king_moves_iterator_finished = true;
            }
        }

        // lazy calculate whether we are in check.
        // let is_check = self.king_moves_iterator.is_check(self.player, self.board.kings[self.player.as_index()].0);
        // println!("Finished calculating is_check: {is_check}");

        if let Some(pawn_moves) = &mut self.pawn_moves_iterator {
            let next_pawn_move = pawn_moves.next();
            if next_pawn_move.is_some() {
                return next_pawn_move;
            } else {
                self.pawn_moves_iterator = None;
            }
        }

        if let Some(knight_moves) = &mut self.knight_moves_iterator {
            let next_knight_move = knight_moves.next();
            if next_knight_move.is_some() {
                return next_knight_move;
            } else {
                self.knight_moves_iterator = None;
            }
        }

        if let Some(bishop_moves) = &mut self.bishop_moves_iterator {
            let next_bishop_move = bishop_moves.next();
            if next_bishop_move.is_some() {
                return next_bishop_move;
            } else {
                self.bishop_moves_iterator = None;
            }
        }

        if let Some(rook_moves) = &mut self.rook_moves_iterator {
            let next_rook_move = rook_moves.next();
            if next_rook_move.is_some() {
                return next_rook_move;
            } else {
                self.rook_moves_iterator = None;
            }
        }

        if let Some(queen_moves) = &mut self.queen_moves_iterator {
            let next_queen_move = queen_moves.next();
            if next_queen_move.is_some() {
                return next_queen_move;
            } else {
                self.queen_moves_iterator = None;
            }
        }

        return None;
    }
}