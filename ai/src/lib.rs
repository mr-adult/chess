use chess_core::Board;
use evaluators::SimpleEvaluator;
use search_algorithms::IterativeDeepeningMovesIterator;

mod search_algorithms;
mod evaluators;

pub(crate) use search_algorithms::SearchAlgorithm;

pub fn iterative_deepening_basic(position: &mut Board, search_depth: usize) -> SimpleEvaluator<IterativeDeepeningMovesIterator> {
    SimpleEvaluator::new(
        IterativeDeepeningMovesIterator::new(position, search_depth)
    )
}
