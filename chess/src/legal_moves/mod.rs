use arr_deque::ArrDeque;
use chess_common::{Location, Player, Rank};

use crate::{chess_move::PossibleMove, Board, Move};
mod bishop;
mod king;
mod knight;
mod pawn;
mod queen;
pub(crate) mod rook;

use bishop::LegalBishopMovesIterator;
use king::{CheckStoppingSquaresIterator, KingProtectingLocationsIterator, LegalKingMovesIterator};
use knight::LegalKnightMovesIterator;
use pawn::LegalPawnMovesIterator;
use queen::LegalQueenMovesIterator;
use rook::LegalRookMovesIterator;

pub struct LegalMovesIterator<'board> {
    board: &'board Board,
    player: Player,
    pawn_moves_iterator: Option<LegalPawnMovesIterator<'board>>,
    knight_moves_iterator: Option<LegalKnightMovesIterator<'board>>,
    bishop_moves_iterator: Option<LegalBishopMovesIterator<'board>>,
    rook_moves_iterator: Option<LegalRookMovesIterator<'board>>,
    queen_moves_iterator: Option<LegalQueenMovesIterator<'board>>,
    king_moves_iterator: LegalKingMovesIterator<'board>,
    king_moves_iterator_finished: bool,
    check_blocking_squares: Option<ArrDeque<Location, 8>>,
    king_protecting_squares: Option<ArrDeque<Location, 8>>,
}

impl<'board> LegalMovesIterator<'board> {
    pub(crate) fn for_board(board: &'board Board) -> Self {
        let player_to_move = board.get_player_to_move();
        Self {
            board,
            player: player_to_move,
            pawn_moves_iterator: Some(LegalPawnMovesIterator::new(board)),
            knight_moves_iterator: Some(LegalKnightMovesIterator::new(board)),
            bishop_moves_iterator: Some(LegalBishopMovesIterator::new(&board)),
            rook_moves_iterator: Some(LegalRookMovesIterator::new(board)),
            queen_moves_iterator: Some(LegalQueenMovesIterator::new(board)),
            king_moves_iterator: LegalKingMovesIterator::new(board, player_to_move),
            king_moves_iterator_finished: false,
            check_blocking_squares: None,
            king_protecting_squares: None,
        }
    }

    fn get_next_move_that_meets_check_constraints<T: Iterator<Item = Move>>(
        king_protecting: &ArrDeque<Location, 8>,
        check_blocks: &Option<ArrDeque<Location, 8>>,
        iter: &mut T,
    ) -> Option<Move> {
        while let Some(move_) = Self::get_next_move_that_blocks_check(check_blocks, iter) {
            if king_protecting
                .iter()
                .any(|protecting| move_.from == *protecting)
            {
                continue;
            }
            return Some(move_);
        }
        return None;
    }

    fn get_next_move_that_blocks_check<T: Iterator<Item = Move>>(
        check_blocks: &Option<ArrDeque<Location, 8>>,
        iter: &mut T,
    ) -> Option<Move> {
        while let Some(next_move) = iter.next() {
            if let Some(blocking) = check_blocks.as_ref() {
                if blocking.iter().any(|loc| *loc == next_move.to) {
                    return Some(next_move);
                }
            } else {
                return Some(next_move);
            }
        }

        return None;
    }
}

impl<'board> Iterator for LegalMovesIterator<'board> {
    type Item = PossibleMove;

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
            match next_king_move {
                None => self.king_moves_iterator_finished = true,
                Some(move_) => return Some(PossibleMove::Normal { move_ }),
            }
        }

        if self.check_blocking_squares.is_none()
            && self
                .king_moves_iterator
                .is_check(self.player, self.board.kings[self.player.as_index()].0)
        {
            self.check_blocking_squares = Some(
                CheckStoppingSquaresIterator::new(
                    self.board,
                    self.player,
                    self.board.kings[self.player.as_index()].0,
                )
                .collect(),
            )
        }

        if self.king_protecting_squares.is_none() {
            self.king_protecting_squares = Some(
                KingProtectingLocationsIterator::new(
                    self.board,
                    self.player,
                    self.board.kings[self.player.as_index()].0,
                )
                .collect(),
            );
        }

        if let Some(pawn_moves) = &mut self.pawn_moves_iterator {
            let move_to_consider = Self::get_next_move_that_meets_check_constraints(
                &self.king_protecting_squares.as_ref().unwrap(),
                &self.check_blocking_squares,
                pawn_moves,
            );

            match move_to_consider {
                None => self.pawn_moves_iterator = None,
                Some(move_) => {
                    if move_.to.rank() == Rank::One || move_.to.rank() == Rank::Eight {
                        return Some(PossibleMove::Promotion { move_ });
                    } else {
                        return Some(PossibleMove::Normal { move_ });
                    }
                }
            }
        }

        if let Some(knight_moves) = &mut self.knight_moves_iterator {
            let move_to_consider = Self::get_next_move_that_meets_check_constraints(
                &self.king_protecting_squares.as_ref().unwrap(),
                &self.check_blocking_squares,
                knight_moves,
            );

            match move_to_consider {
                None => self.knight_moves_iterator = None,
                Some(move_) => return Some(PossibleMove::Normal { move_ }),
            }
        }

        if let Some(bishop_moves) = &mut self.bishop_moves_iterator {
            let move_to_consider = Self::get_next_move_that_meets_check_constraints(
                &self.king_protecting_squares.as_ref().unwrap(),
                &self.check_blocking_squares,
                bishop_moves,
            );

            match move_to_consider {
                None => self.bishop_moves_iterator = None,
                Some(move_) => return Some(PossibleMove::Normal { move_ }),
            }
        }

        if let Some(rook_moves) = &mut self.rook_moves_iterator {
            let move_to_consider = Self::get_next_move_that_meets_check_constraints(
                &self.king_protecting_squares.as_ref().unwrap(),
                &self.check_blocking_squares,
                rook_moves,
            );

            match move_to_consider {
                None => self.rook_moves_iterator = None,
                Some(move_) => return Some(PossibleMove::Normal { move_ }),
            }
        }

        if let Some(queen_moves) = &mut self.queen_moves_iterator {
            let move_to_consider = Self::get_next_move_that_meets_check_constraints(
                &self.king_protecting_squares.as_ref().unwrap(),
                &self.check_blocking_squares,
                queen_moves,
            );

            match move_to_consider {
                None => self.queen_moves_iterator = None,
                Some(move_) => return Some(PossibleMove::Normal { move_ }),
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

        let actual_legal_moves = Board::default()
            .legal_moves()
            .map(|possible_move| possible_move.move_().clone())
            .collect::<HashSet<_>>();
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
                .map(|possible_move| possible_move.move_().clone())
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
                .map(|possible_move| possible_move.move_().clone())
                .filter(|move_| move_.from == Location::new(File::g, Rank::Eight))
                .collect::<HashSet<_>>();

        assert_move_sets_equal(&expected_legal_moves_from_g8, &actual_legal_moves_from_g8);
    }

    #[test]
    fn white_in_check_can_only_make_moves_to_exit_check() {
        let expected_moves = [
            // king move out of check
            Move {
                from: Location::new(File::e, Rank::One),
                to: Location::new(File::e, Rank::Two),
            },
            // pawn moves to block the queen
            Move {
                from: Location::new(File::b, Rank::Two),
                to: Location::new(File::b, Rank::Four),
            },
            Move {
                from: Location::new(File::c, Rank::Two),
                to: Location::new(File::c, Rank::Three),
            },
            // knight blocks
            Move {
                from: Location::new(File::b, Rank::One),
                to: Location::new(File::d, Rank::Two),
            },
            Move {
                from: Location::new(File::b, Rank::One),
                to: Location::new(File::c, Rank::Three),
            },
            // bishop block
            Move {
                from: Location::new(File::c, Rank::One),
                to: Location::new(File::d, Rank::Two),
            },
            // queen block
            Move {
                from: Location::new(File::d, Rank::One),
                to: Location::new(File::d, Rank::Two),
            },
        ]
        .into_iter()
        .collect::<HashSet<_>>();

        let actual_moves =
            Board::from_str("rnb1kbnr/pp1ppppp/8/q1p5/3P4/4P3/PPP2PPP/RNBQKBNR w KQkq - 4 3")
                .unwrap()
                .legal_moves()
                .map(|possible_move| possible_move.move_().clone())
                .collect::<HashSet<_>>();
        assert_move_sets_equal(&expected_moves, &actual_moves)
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
