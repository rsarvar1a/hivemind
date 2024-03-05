use arrayvec::ArrayVec;

use crate::prelude::*;

#[derive(Clone, Debug, Default)]
/// A particular line taken by the evaluator, which is a continuation and a corresponding score.
pub struct Variation
{
    pub moves: ArrayVec<Move, { scalars::MAXIMUM_PLY }>,
    pub score: i32,
}

impl Variation
{
    pub fn load(&mut self, mv: Move, rest: &Variation)
    {
        self.moves.clear();
        self.moves.push(mv);
        self.moves.try_extend_from_slice(&rest.moves).unwrap();
    }
}
