use std::marker::PhantomData;

use crate::{Board, PossibleMovesIterator};
use chess_parsers::PieceMove;
use streaming_iterator::StreamingIterator;

pub struct IterativeDeepeningMovesIterator<'board> {
    /// The phantom data of the board to satisfy Rust's
    /// borrowing constraints. There is no way for rust to
    /// validate that we are using the board correctly, but
    /// we can guarantee that the board will never be edited
    /// while we have a reference to it given out.
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
            done: max_depth == 0,
        }
    }

    pub fn board(&self) -> &Board {
        unsafe { &*self.board }
    }

    pub fn current_move_stack(&self) -> Vec<PieceMove> {
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

    pub fn current_depth(&self) -> usize {
        self.moves_iter_stack.len()
    }
}

impl<'board> StreamingIterator for IterativeDeepeningMovesIterator<'board> {
    type Item = Board;

    fn advance(&mut self) {
        if self.done {
            return;
        }
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

unsafe impl<'board> Send for IterativeDeepeningMovesIterator<'board> {}

impl<'board> Drop for IterativeDeepeningMovesIterator<'board> {
    fn drop(&mut self) {
        if !self.top_iter_made_move {
            self.moves_iter_stack.pop();
        }

        while let Some(_) = self.moves_iter_stack.pop() {
            unsafe { &mut *self.board }.undo().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        io::{stdout, Write},
    };

    use crate::Board;
    use streaming_iterator::StreamingIterator;

    use super::IterativeDeepeningMovesIterator;

    use whitespacesv::{ColumnAlignment, WSVWriter};

    #[test]
    fn starting_position_perft() {
        let values: Vec<usize> = vec![
            1,
            20,
            400,
            8_902,
            197_281,
            4_865_609,
            119_060_324,
            3_195_901_860,
            84_998_978_956,
            2_439_530_234_167,
            69_352_859_712_417,
        ];

        let mut board = Board::default();
        const depth: u8 = 4;
        let perft_nodes = perft_nodes(&mut board, depth);
        let mut final_printout = String::new();

        for (i, depth_map) in perft_nodes.into_iter().enumerate() {
            let mut final_touch = String::new();
            let mut sorted_depth_map = depth_map.into_iter().collect::<Vec<_>>();
            sorted_depth_map.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

            if i == depth as usize - 1 {
                let mut condensed = HashMap::new();
                for (k, v) in sorted_depth_map.iter() {
                    let key = k.split(" ").next().unwrap();
                    if let Some(val) = condensed.get_mut(key) {
                        *val += *v;
                    } else {
                        condensed.insert(key, *v);
                    }
                }

                let mut condensed_sorted = condensed.into_iter().collect::<Vec<_>>();
                condensed_sorted.sort_by(|&(k1, _), &(k2, _)| k1.cmp(k2));

                final_touch.push_str(&format!("Condensed results at depth: {}:\n", i + 1));
                let wsv = WSVWriter::new(
                    condensed_sorted
                        .into_iter()
                        .map(|(k, v)| [Some(k.to_string()), Some(v.to_string())]),
                )
                .align_columns(ColumnAlignment::Left)
                .to_string();

                final_touch.push('\n');
                final_touch.push_str(&wsv);
                final_touch.push('\n');
            }

            final_printout.push_str(&format!("Results at depth: {}:\n", i + 1));
            let mut total = 0;
            let wsv = WSVWriter::new(sorted_depth_map.into_iter().map(|(k, v)| {
                total += v;
                [Some(k), Some(v.to_string())]
            }))
            .align_columns(ColumnAlignment::Left)
            .to_string();

            final_printout.push_str(&wsv);
            final_printout.push('\n');

            if final_touch.len() > 0 {
                final_printout.push_str(&final_touch);
            }

            final_printout.push_str(&format!("\nTotal: {}\n", total));
            final_printout.push_str("\n\n");
        }

        let mut stdout = stdout().lock();
        stdout.write_all(final_printout.as_bytes()).ok();
        stdout.flush().ok();
    }

    fn perft_nodes(board: &mut Board, depth: u8) -> Vec<HashMap<String, usize>> {
        let mut maps = Vec::with_capacity(depth as usize - 1);
        for _ in 0..depth {
            maps.push(HashMap::new());
        }
        let mut stream = IterativeDeepeningMovesIterator::new(board, depth as usize);
        let mut len = 0;
        while let Some(board) = stream.next() {
            let half_moves = board.half_moves_played() as usize;
            let map = maps.get_mut(half_moves - 1).unwrap();

            let acn = board.get_move_history_acn();
            let acn_minus_1 = &acn[0..acn.len() - 1];
            let acn_str = acn_minus_1
                .into_iter()
                .map(|acn| acn.to_string())
                .collect::<Vec<_>>()
                .join(" ");

            if let Some(value) = map.get_mut(&acn_str) {
                *value += 1;
            } else {
                map.insert(acn_str, 1);
            }

            if board.half_moves_played() != depth {
                continue;
            }

            len += 1;
        }

        assert!(
            len == maps
                .get_mut(depth as usize - 1)
                .unwrap()
                .values()
                .fold(0, |agg, val| { agg + val })
        );
        maps
    }
}