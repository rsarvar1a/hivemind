use crate::prelude::*;

#[derive(Clone, Debug, Default)]
/// An evaluator with absolutely no policy.
///
/// This evaluator is only useful for move generation, and should **not** be used for anything else!
pub struct BasicEvaluator;

impl Evaluator for BasicEvaluator
{
    type Generator = BasicMoveGenerator;

    fn best_move(&mut self, board: &Board, _args: SearchArgs) -> Move
    {
        let mut movegen = self.generate_moves(board);

        match movegen.next()
        {
            | Some(mv) => mv,
            | None => Move::Pass,
        }
    }

    fn generate_moves(&self, board: &Board) -> Self::Generator
    {
        BasicMoveGenerator::new(board)
    }

    fn new(_options: UhpOptions) -> Self
    {
        BasicEvaluator::default()
    }
}

/// A move generator with no policy whatsoever. It is also not lazy!
pub struct BasicMoveGenerator
{
    moves: Vec<Move>,
    index: usize
}

impl Iterator for BasicMoveGenerator
{
    type Item = Move;
    fn next(&mut self) -> Option<Move>
    {
        let next = self.moves.get(self.index);
        self.index += 1;
        next.copied()
    }
}

impl BasicMoveGenerator
{
    pub fn new(board: &Board) -> Self
    {
        BasicMoveGenerator
        {
            moves: board.generate_moves(),
            index: 0
        }
    }
}
