use crate::prelude::*;

#[derive(Clone, Debug, Default)]
/// An evaluator with absolutely no policy.
///
/// This evaluator is only useful for lazy move generation, and should **not** be used for anything else!
pub struct BasicEvaluator<'a>
{
    _marker: std::marker::PhantomData<&'a Self>,
}

impl<'a> Evaluator<'a> for BasicEvaluator<'a>
{
    type Generator = BasicMoveGenerator<'a>;

    fn best_move(&self, _board: &Board, _args: SearchArgs) -> Move
    {
        // This one is *really** not implemented. Don't use it!
        let _ = Error::not_implemented();
        Move::Pass
    }

    fn generate_moves(&self, board: &'a Board) -> Self::Generator
    {
        BasicMoveGenerator::new(board)
    }

    fn new(_options: UhpOptions) -> Self
    {
        BasicEvaluator::default()
    }
}

/// A lazy iterator with no policy whatsoever.
pub struct BasicMoveGenerator<'a>
{
    board: &'a Board,
}

impl<'a> Iterator for BasicMoveGenerator<'a>
{
    type Item = Move;
    fn next(&mut self) -> Option<Move>
    {
        Error::not_implemented();
        None
    }
}

impl<'a> BasicMoveGenerator<'a>
{
    fn new(board: &'a Board) -> Self
    {
        BasicMoveGenerator { board }
    }
}
