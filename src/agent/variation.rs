use arrayvec::ArrayVec;

use crate::prelude::*;

#[derive(Clone, Debug, Default)]
/// A particular line taken by the evaluator, which is a continuation and a corresponding score.
pub struct Variation
{
    pub moves: ArrayVec<Move, { scalars::MAXIMUM_PLY }>,
    pub score: i32,
}
