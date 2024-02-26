use crate::prelude::*;

pub struct StrongestEvaluator<'a>
{
    marker: std::marker::PhantomData<&'a Self>,
    ttable: TranspositionTable,
}

impl<'a> Evaluator<'a> for StrongestEvaluator<'a>
{
    type Generator = PrioritizingMoveGenerator<'a>;

    fn best_move(&self, board: &Board, args: SearchArgs) -> Move
    {
        let _unused = (board, args);
        let _ = Error::not_implemented();
        Move::Pass
    }

    fn generate_moves(&self, board: &'a Board) -> Self::Generator
    {
        PrioritizingMoveGenerator::new(board)
    }

    fn new(options: UhpOptions) -> Self
    {
        let bytes = (options.memory * 1e+9) as usize;
        StrongestEvaluator {
            marker: std::marker::PhantomData,
            ttable: TranspositionTable::new(bytes),
        }
    }
}

/// A move generator that tries to generate better moves first for the purposes of
/// making position evaluation more efficient.
pub struct PrioritizingMoveGenerator<'a>
{
    board: &'a Board,
}

impl<'a> Iterator for PrioritizingMoveGenerator<'a>
{
    type Item = Move;
    fn next(&mut self) -> Option<Self::Item>
    {
        None
    }
}

impl<'a> PrioritizingMoveGenerator<'a>
{
    pub fn new(board: &'a Board) -> Self
    {
        PrioritizingMoveGenerator { board }
    }
}
