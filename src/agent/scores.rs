mod consts
{
    pub const MINIMUM_WIN: i32 = i16::MAX as i32;
}

use consts::*;

/// Embed the ply into winning or losing scores.
pub fn normalize(score: i32, ply: usize) -> i32
{
    if score >= MINIMUM_WIN
    {
        score - ply as i32
    }
    else if score <= -MINIMUM_WIN
    {
        score + ply as i32
    }
    else
    {
        score
    }
}

/// Extract the score from an embedded score.
pub fn reconstruct(score: i32, ply: usize) -> i32
{
    if score >= MINIMUM_WIN
    {
        score + ply as i32
    }
    else if score <= -MINIMUM_WIN
    {
        score - ply as i32
    }
    else
    {
        score
    }
}
