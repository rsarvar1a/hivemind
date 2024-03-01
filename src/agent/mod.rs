use crate::prelude::*;

pub mod evaluators;
pub mod scalars;
pub mod search;
pub mod table;
pub mod variation;

pub use scalars::*;
pub use search::*;
pub use table::*;
pub use variation::*;

/// A trait representing a collection of policies by which we can evaluate a board position and find the best continuations.
pub trait Evaluator
{
    type Generator: MoveGenerator;

    /// Returns the best move in the current position.
    fn best_move(&mut self, board: &Board, args: SearchArgs) -> Move;

    /// Generates all valid moves on the given board.
    /// For performance reasons, this should be as lazy as possible!
    fn generate_moves(&self, board: &Board) -> Self::Generator;

    /// Returns a new evaluator. Evaluators should be instanced so that they can support internal state.
    fn new(options: UhpOptions) -> Self;
}

/// A trait alias that represents a forward iterator on a collection of moves.
pub trait MoveGenerator = Iterator;
