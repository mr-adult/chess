use arr_deque::ArrDeque;
use chess_common::{black, white, Location, Player};

use crate::{Board, Move};
mod bishop;
mod king;
mod knight;
mod pawn;
mod queen;
pub(crate) mod rook;

use bishop::LegalBishopMovesIterator;
use king::LegalKingMovesIterator;
use knight::LegalKnightMovesIterator;
use pawn::LegalPawnMovesIterator;
use queen::LegalQueenMovesIterator;
use rook::LegalRookMovesIterator;

pub(crate) struct LegalMovesIterator<'board> {
    board: &'board Board,
    player: Player,
    is_check: Option<bool>,
    pawn_moves_iterator: Option<LegalPawnMovesIterator<'board>>,
    knight_moves_iterator: Option<LegalKnightMovesIterator<'board>>,
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

#[cfg(test)]
mod tests {
    use chess_common::{File, Location, Rank};
    use std::{collections::HashSet, str::FromStr};

    use crate::{Board, Move};

    #[test]
    fn default_starting_position() {
        let pawn_moves = File::all_files_ascending().flat_map(|file| {
            [
                Move {
                    from: Location::new(file, Rank::Two),
                    to: Location::new(file, Rank::Three),
                },
                Move {
                    from: Location::new(file, Rank::Two),
                    to: Location::new(file, Rank::Four),
                },
            ]
        });

        let knight_one_moves = [File::a, File::c].into_iter().map(|file| Move {
            from: Location::new(File::b, Rank::One),
            to: Location::new(file, Rank::Three),
        });

        let knight_two_moves = [File::f, File::h].into_iter().map(|file| Move {
            from: Location::new(File::g, Rank::One),
            to: Location::new(file, Rank::Three),
        });

        let expected_legal_moves = pawn_moves
            .chain(knight_one_moves)
            .chain(knight_two_moves)
            .collect::<HashSet<_>>();

        let actual_legal_moves = Board::default().legal_moves().collect::<HashSet<_>>();
        assert_move_sets_equal(&expected_legal_moves, &actual_legal_moves);
    }

    #[test]
    fn starting_position_black_move() {
        let pawn_moves = File::all_files_ascending().flat_map(|file| {
            [
                Move {
                    from: Location::new(file, Rank::Seven),
                    to: Location::new(file, Rank::Six),
                },
                Move {
                    from: Location::new(file, Rank::Seven),
                    to: Location::new(file, Rank::Five),
                },
            ]
        });

        let knight_one_moves = [File::a, File::c].into_iter().map(|file| Move {
            from: Location::new(File::b, Rank::Eight),
            to: Location::new(file, Rank::Six),
        });

        let knight_two_moves = [File::f, File::h].into_iter().map(|file| Move {
            from: Location::new(File::g, Rank::Eight),
            to: Location::new(file, Rank::Six),
        });

        let expected_legal_moves = pawn_moves
            .chain(knight_one_moves)
            .chain(knight_two_moves)
            .collect::<HashSet<_>>();

        let actual_legal_moves =
            Board::from_str("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 1 1")
                .unwrap()
                .legal_moves()
                .collect::<HashSet<_>>();

        assert_move_sets_equal(&expected_legal_moves, &actual_legal_moves);
    }

    #[test]
    fn black_down_left_knight_move() {
        let g8 = Location::new(File::g, Rank::Eight);
        let expected_legal_moves_from_g8 = [
            Location::new(File::h, Rank::Six),
            Location::new(File::f, Rank::Six),
            Location::new(File::e, Rank::Seven),
        ]
        .into_iter()
        .map(|to| Move {
            from: g8.clone(),
            to,
        })
        .collect::<HashSet<_>>();

        let actual_legal_moves_from_g8 =
            Board::from_str("rnbqkbnr/pppp1ppp/8/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR b KQkq - 3 2")
                .unwrap()
                .legal_moves()
                .filter(|move_| move_.from == Location::new(File::g, Rank::Eight))
                .collect::<HashSet<_>>();

        assert_move_sets_equal(&expected_legal_moves_from_g8, &actual_legal_moves_from_g8);
    }

    fn assert_move_sets_equal(expected: &HashSet<Move>, actual: &HashSet<Move>) {
        if expected == actual {
            return;
        }

        let mut messages = Vec::with_capacity(1);
        for item in expected {
            if !actual.contains(item) {
                messages.push(format!(
                    "Expected the move {:?}, but it was not found.",
                    item
                ))
            }
        }

        for item in actual {
            if !expected.contains(item) {
                messages.push(format!(
                    "Found the move {:?}, but it was not expected.",
                    item
                ));
            }
        }

        panic!("{}", messages.join("\n"));
    }
}
