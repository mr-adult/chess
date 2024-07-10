mod iterative_deepening;

use chess_parsers::PieceMove;
pub use iterative_deepening::IterativeDeepeningMovesIterator;
use streaming_iterator::StreamingIterator;

pub trait SearchAlgorithm: StreamingIterator {
    fn current_move_stack(&self) -> Vec<PieceMove>;
    fn current_depth(&self) -> usize;
}