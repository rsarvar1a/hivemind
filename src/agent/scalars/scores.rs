use crate::prelude::*;

pub mod consts
{
    use super::*;
    pub const MINIMUM_WIN: i32 = i16::MAX as i32 - scalars::MAXIMUM_PLY as i32;
    pub const MINIMUM_LOSS: i32 = -MINIMUM_WIN;
}

pub use consts::*;

/// Embed the ply into winning or losing scores.
pub fn normalize(score: i32) -> i32
{
    if score > MINIMUM_WIN - MAXIMUM_PLY as i32
    {
        score - 1
    }
    else if score < MINIMUM_LOSS + MAXIMUM_PLY as i32
    {
        score + 1
    }
    else
    {
        score
    }
}

/// Extract the score from an embedded score.
pub fn reconstruct(score: i32) -> i32
{
    if score > MINIMUM_WIN - MAXIMUM_PLY as i32
    {
        MINIMUM_WIN
    }
    else if score < MINIMUM_LOSS + MAXIMUM_PLY as i32
    {
        MINIMUM_LOSS
    }
    else
    {
        score
    }
}