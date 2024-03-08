use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

use dashmap::DashMap;

use crate::prelude::*;

mod entry;
mod token;

pub use entry::{TTAge, TTBound, TTEntry, TTEntryData, TTHit};
pub use token::MoveToken;

#[derive(Debug)]
/// A lockfree, concurrent implementation of a transposition table.
pub struct TranspositionTable
{
    map: Arc<DashMap<u128, TTEntryData>>,
    age: AtomicU8,
    cap: usize,
}

/// The sort of reference we get into the dashmap, but we want to hold onto it as little as possible.
type TTRef<'a> = dashmap::mapref::one::Ref<'a, u128, TTEntryData>;

impl TranspositionTable
{
    /// The upper bound on the table's age.
    const EXTENT_AGE: u8 = 0x3F;

    /// TT-reduction weight.
    pub const DEPTH_DECREMENT_THRESHOLD: Depth = Depth::new(4);

    /// Checks if the score here is any good.
    pub fn check(&self, key: ZobristHash, depth: Depth, candidate: &mut Option<Move>, a: &mut i32, b: &mut i32) -> Option<i32>
    {
        if let Some(hit) = self.load(key)
        {
            *candidate = hit.mv.into();

            if hit.depth >= depth
            {
                match hit.bound
                {
                    TTBound::Exact =>
                    {
                        return Some(hit.score);
                    },
                    TTBound::Lower => 
                    {
                        *a = (*a).max(hit.score);
                    },
                    TTBound::Upper =>
                    {
                        *b = (*b).min(hit.score);
                    },
                    _ => unreachable!()
                };

                if *a >= *b
                {
                    return Some(hit.score);
                }
            }
        }
        None
    }

    /// Loads a variation from the table.
    pub fn get_principal_variation(&self, board: &Board, variation: &mut Variation)
    {
        variation.moves.clear();
        variation.score = 0;

        let mut history = Vec::new();
        let mut board = board.clone();
        let mut zobrist = board.zobrist();

        while let Some(hit) = self.load(zobrist)
        {
            let mv: Move = Option::<Move>::from(hit.mv).unwrap_or(Move::Pass);
            variation.moves.push(ScoredMove { mv, score: hit.score });

            board.play_unchecked(&mv);
            zobrist = board.zobrist();

            if history.contains(&zobrist)
            {
                break;
            }
            history.push(zobrist);
        }

        if !variation.moves.is_empty()
        {
            variation.score = variation.moves[0].score;
        }
    }

    /// Increments the age of the table.
    pub fn increment(&self)
    {
        let new = Self::EXTENT_AGE & (self.age.load(Ordering::Relaxed) + 1);
        self.age.store(new, Ordering::Relaxed);
    }

    /// Finds the hitinfo associated with this board state, if one exists.
    pub fn load(&self, key: ZobristHash) -> Option<TTHit>
    {
        self.get(&key).map(|e| {
            let entry: TTEntry = e.to_owned().into();

            TTHit {
                key:   entry.key,
                mv:    entry.mv,
                depth: entry.depth,
                bound: entry.age.bound,
                score: entry.score,
            }
        })
    }

    /// Creates a new transposition table with the given memory constraints.
    pub fn new(bytes: usize) -> TranspositionTable
    {
        // Get the number of entries that fit in our table.
        let cap = bytes / TTEntry::SIZE;
        log::trace!("Allocated a TranspositionTable with {} entries. ({} bytes)", cap, bytes);

        TranspositionTable {
            map: Arc::new(DashMap::with_capacity(cap)),
            age: AtomicU8::new(0),
            cap,
        }
    }

    /// Stores a new evaluation into the transposition table.
    pub fn store(&self, entry: &TTEntry)
    {
        let mut entry = *entry;
        let existing: Option<TTEntry> = self.get(&entry.key).map(|e| e.to_owned().into());

        match existing
        {
            | Some(prev) =>
            {
                entry.mv = if entry.mv.is_some() { entry.mv } else { prev.mv };

                if entry.key != prev.key
                    || entry.age.bound == TTBound::Exact && prev.age.bound != TTBound::Exact
                    || self.should_overwrite(&prev, &entry)
                {
                    let data: TTEntryData = entry.into();
                    self.map.insert(entry.key, data);
                }
            }
            | None =>
            {
                let data: TTEntryData = entry.into();
                self.put(&entry.key, data);
            }
        };
    }
}

/// Private mapping implementation for the table.
impl TranspositionTable
{
    /// Returns the key modulo the maximum number of entries, avoiding reallocation.
    fn capacity_hash(&self, key: &ZobristHash) -> u128
    {
        key % (self.cap as u128)
    }

    /// Gets an entry from the table, ensuring we don't overdo the capacity.
    fn get(&self, key: &ZobristHash) -> Option<TTRef<'_>>
    {
        let meta_key = self.capacity_hash(key);
        self.map.get(&meta_key)
    }

    /// Puts an entry into the table, ensuring we don't overdo the capacity.
    fn put(&self, key: &ZobristHash, data: TTEntryData)
    {
        let meta_key = self.capacity_hash(key);
        self.map.insert(meta_key, data);
    }

    /// Whether or not to overwrite an entry based on age priority.
    fn should_overwrite(&self, prev: &TTEntry, next: &TTEntry) -> bool
    {
        prev.age.age != self.age.load(Ordering::SeqCst) || prev.depth <= next.depth
    }
}

/// Because the LSP isn't giving me confidence that this is true!
unsafe impl std::marker::Sync for TranspositionTable {}
