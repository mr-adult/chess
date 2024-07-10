use std::marker::PhantomData;

use chess_core::{Board, PossibleMovesIterator};
use chess_parsers::PieceMove;
use streaming_iterator::StreamingIterator;

use crate::SearchAlgorithm;

pub struct IterativeDeepeningMovesIterator<'board> {
    board_phantom: PhantomData<&'board Board>,
    /// The actual pointer with lifetime 'board.
    board: *mut Board,
    current_target_depth: usize,
    top_iter_made_move: bool,
    moves_iter_stack: Vec<PossibleMovesIterator<'board>>,
    found_new_move_at_depth: bool,
    max_depth: usize,
    done: bool,
}

impl<'board> IterativeDeepeningMovesIterator<'board> {
    pub(crate) fn new(board: &'board mut Board, max_depth: usize) -> Self {
        Self {
            board_phantom: PhantomData {},
            board,
            current_target_depth: 1,
            top_iter_made_move: false,
            moves_iter_stack: vec![board.possible_moves()],
            found_new_move_at_depth: true,
            max_depth,
            done: false,
        }
    }
}

impl<'board> StreamingIterator for IterativeDeepeningMovesIterator<'board> {
    type Item = Board;

    fn advance(&mut self) {
        if self.done { return; }
        loop {
            if self.moves_iter_stack.len() == 0 {
                if !self.found_new_move_at_depth || self.current_target_depth == self.max_depth {
                    self.done = true;
                    break;
                }

                self.found_new_move_at_depth = false;
                self.top_iter_made_move = false;
                self.current_target_depth += 1;
                self.moves_iter_stack
                    .push(unsafe { &mut *self.board }.possible_moves());
            }

            let stack_len = self.moves_iter_stack.len();
            match self.moves_iter_stack.get_mut(stack_len - 1) {
                None => break,
                Some(top) => {
                    if self.top_iter_made_move {
                        assert!(unsafe { &mut *self.board }.undo().is_ok());
                    }

                    match top.next() {
                        None => {
                            self.moves_iter_stack.pop();
                            self.top_iter_made_move = !self.moves_iter_stack.is_empty();
                        }
                        Some(value) => {
                            self.top_iter_made_move = true;
                            let board_mut = unsafe { &mut *self.board };
                            assert!(board_mut.make_move(value).is_ok());

                            if self.current_target_depth == self.moves_iter_stack.len() {
                                self.found_new_move_at_depth = true;
                                break;
                            }

                            self.moves_iter_stack.push(board_mut.possible_moves());
                            self.top_iter_made_move = false;
                        }
                    }
                }
            }
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.done {
            return None;
        }

        unsafe { Some(&*self.board) }
    }
}

impl<'board> SearchAlgorithm for IterativeDeepeningMovesIterator<'board> {
    fn current_move_stack(&self) -> Vec<PieceMove> {
        let mut all_moves = unsafe { &*self.board }.get_move_history_acn();

        let net_new_moves_len = if self.top_iter_made_move {
            self.moves_iter_stack.len()
        } else {
            if self.moves_iter_stack.len() == 0 {
                0
            } else {
                self.moves_iter_stack.len() - 1
            }
        };
        let mut net_new_moves = Vec::with_capacity(net_new_moves_len);
        let mut temp = Vec::with_capacity(net_new_moves_len);

        for _ in 0..net_new_moves_len {
            temp.push(all_moves.pop().expect("Never to fail"));
        }

        for _ in 0..net_new_moves_len {
            net_new_moves.push(temp.pop().expect("Never to fail"));
        }

        net_new_moves
    }

    fn current_depth(&self) -> usize {
        self.moves_iter_stack.len()
    }
}

impl<'board> Drop for IterativeDeepeningMovesIterator<'board> {
    fn drop(&mut self) {
        if self.top_iter_made_move {
            self.moves_iter_stack.pop();
            unsafe { &mut *self.board }.undo().ok();
        }

        while let Some(_) = self.moves_iter_stack.pop() {
            unsafe { &mut *self.board }.undo().ok();
        }
    }
}
