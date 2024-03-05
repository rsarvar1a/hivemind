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

    ///
    pub const DEPTH_DECREMENT_THRESHOLD: Depth = Depth::new(4);

    /// Increments the age of the table.
    pub fn increment(&self)
    {
        let new = Self::EXTENT_AGE & (self.age.load(Ordering::Relaxed) + 1);
        self.age.store(new, Ordering::Relaxed);
    }

    /// Finds the hitinfo associated with this board state, if one exists.
    pub fn load(&self, key: ZobristHash, ply: usize) -> Option<TTHit>
    {
        self.get(&key).map(|e| {
            let entry: TTEntry = e.to_owned().into();

            TTHit {
                key:   entry.key,
                mv:    entry.mv,
                depth: entry.depth,
                bound: entry.age.bound,
                value: scores::reconstruct(entry.score, ply),
                eval:  entry.eval,
            }
        })
    }

    /// Creates a new transposition table with the given memory constraints.
    pub fn new(bytes: usize) -> TranspositionTable
    {
        // Get the number of entries that fit in our table.
        let cap = bytes / TTEntry::SIZE;
        log::debug!("Allocated a TranspositionTable with {} entries. ({} bytes)", cap, bytes);

        TranspositionTable {
            map: Arc::new(DashMap::with_capacity(cap)),
            age: AtomicU8::new(0),
            cap,
        }
    }

    /// Stores a new evaluation into the transposition table.
    pub fn store(&self, entry: &TTEntry, ply: usize)
    {
        let mut entry = *entry;
        let existing: Option<TTEntry> = self.get(&entry.key).map(|e| e.to_owned().into());

        match existing
        {
            | Some(prev) =>
            {
                entry.mv = if entry.mv.is_some() { entry.mv } else { prev.mv };
                entry.score = scores::normalize(entry.score, ply);

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
        let insert_bonus: i32 = next.age.bound.into();
        let record_bonus: i32 = prev.age.bound.into();

        let aged: i32 = self.age.load(Ordering::Relaxed) as i32;
        let diff: i32 = (aged + 64 - prev.age.age as i32) & Self::EXTENT_AGE as i32;

        let insert_prio: Depth = next.depth + insert_bonus + (diff * diff) / 4;
        let record_prio: Depth = prev.depth + record_bonus;

        insert_prio * 3 >= record_prio * 2
    }
}

/// Because the LSP isn't giving me confidence that this is true!
unsafe impl std::marker::Sync for TranspositionTable {}
