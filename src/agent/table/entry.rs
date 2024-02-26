use crate::prelude::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// The age of a particular entry.
pub struct TTAge
{
    pub age:   u8,
    pub bound: TTBound,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A bound on the age on a TTEntry.
pub enum TTBound
{
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

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Data associated with the most recent evaluation of a particular board state.
pub struct TTEntry
{
    // This entry's main k-v.
    pub key:   ZobristHash, // 16
    pub mv:    MoveToken,   // 4
    pub depth: Depth,       // 4
    pub eval:  i32,         // 4
    pub score: i32,         // 4
    pub age:   TTAge,       // 2
    #[allow(unused)]
    _padding:  [i16; 3], // 6
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
    const __TTENTRY_SIZE_CORRECT: () = assert!(Self::SIZE == 48);
}

/// A type alias representing data that is generated from a TTEntry.
pub type TTEntryData = [u64; 6];

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
    pub bound: TTBound,
    pub value: i32,
    pub eval:  i32,
}
