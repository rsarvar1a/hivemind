use arrayvec::ArrayVec;

use crate::prelude::*;

#[derive(Clone, Debug, Default)]
/// A particular line taken by the evaluator, which is a continuation and a corresponding score.
pub struct Variation
{
    pub moves: ArrayVec<ScoredMove, { scalars::MAXIMUM_PLY }>,
    pub score: i32,
}

impl Variation
{
    pub fn load(&mut self, mv: ScoredMove, rest: &Variation)
    {
        self.moves.clear();
        self.moves.push(mv);
        self.moves.try_extend_from_slice(&rest.moves).unwrap();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A move paired with an evaluation.
pub struct ScoredMove
{
    pub mv: Move,
    pub score: i32
}

impl PartialOrd for ScoredMove
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> 
    {
        self.score.partial_cmp(&other.score)    
    }
}

impl Ord for ScoredMove 
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering 
    {
        self.score.cmp(&other.score)    
    }
}
