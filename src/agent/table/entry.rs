use crate::prelude::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
// The age of a particular entry.
pub struct TTAge
{
    pub age:   u8,
    pub bound: TTBound,
}

impl TTAge
{
    pub fn compute(score: i32, alpha: i32, beta: i32) -> TTAge
    {
        let bound = if score <= alpha
        {
            TTBound::Upper
        }
        else if score >= beta
        {
            TTBound::Lower
        }
        else
        {
            TTBound::Exact
        };

        TTAge { age: 0, bound }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// A bound on the age on a TTEntry.
pub enum TTBound
{
    #[default]
    None  = 0,
    Upper = 1,
    Lower = 2,
    Exact = 3,
}

impl From<TTBound> for i32
{
    fn from(value: TTBound) -> i32
    {
        unsafe { std::mem::transmute::<TTBound, u8>(value) as i32 }
    }
}

#[allow(unused)]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Data associated with the most recent evaluation of a particular board state.
pub struct TTEntry
{
    // This entry's main k-v.
    pub key:   ZobristHash, // 16
    pub mv:    MoveToken,   // 4
    pub depth: Depth,       // 4
    pub score: i32,         // 4
    pub age:   TTAge,       // 2
}

impl From<TTEntryData> for TTEntry
{
    fn from(value: TTEntryData) -> Self
    {
        unsafe { std::mem::transmute(value) }
    }
}

impl TTEntry
{
    /// The size in bytes of an entry.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    #[allow(clippy::assertions_on_constants)]
    const __TTENTRY_SIZE_CORRECT: () = assert!(Self::SIZE == 32);
}

/// A type alias representing data that is generated from a TTEntry.
pub type TTEntryData = [u64; 4];

impl From<TTEntry> for TTEntryData
{
    fn from(value: TTEntry) -> Self
    {
        unsafe { std::mem::transmute(value) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A hit in the transposition table for a particular board state.
pub struct TTHit
{
    pub key:   ZobristHash,
    pub mv:    MoveToken,
    pub depth: Depth,
    pub score: i32,
    pub bound: TTBound,
}
