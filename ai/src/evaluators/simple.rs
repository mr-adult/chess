use chess_common::Player;
use chess_core::Board;
use chess_parsers::PieceMove;
use streaming_iterator::StreamingIterator;

use crate::SearchAlgorithm;

pub struct SimpleEvaluator<T> where T: SearchAlgorithm<Item = Board> {
    search_algo: T,
    current_eval: Option<i32>,
}

impl<T> SimpleEvaluator<T> where T: SearchAlgorithm<Item = Board> {
    pub(crate) fn new(search_algo: T) -> Self {
        Self {
            search_algo,
            current_eval: None,
        }
    }

    pub fn current_move_stack(&self) -> Vec<PieceMove> {
        self.search_algo.current_move_stack()
    }

    pub fn current_depth(&self) -> usize {
        self.search_algo.current_depth()
    }
}

impl<T> StreamingIterator for SimpleEvaluator<T> where T: SearchAlgorithm<Item = Board> {
    type Item = i32;

    fn advance(&mut self) {
        self.search_algo.advance();
        self.current_eval = match self.search_algo.get() {
            None => None,
            Some(board) => {
                if board.is_check_mate() {
                    Some(match board.player_to_move() {
                        Player::White => i32::MAX,
                        Player::Black => i32::MIN,
                    })
                } else {
                    Some(board.material_advantage())
                }
            }
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        self.current_eval.as_ref()
    }
}