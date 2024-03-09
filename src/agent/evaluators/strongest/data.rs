use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
};

use mini_moka::sync::Cache;

use crate::prelude::*;

#[derive(Debug)]
/// Contains any information shared between threads.
pub struct GlobalData
{
    pub args:           SearchArgs,
    pub max_depth:      AtomicU64,
    pub options:        UhpOptions,
    pub start_time:     Instant,
    pub stopped:        AtomicBool,
    pub transpositions: TranspositionTable,
}

impl GlobalData
{
    /// Creates a new GlobalData with the given options.
    pub fn new(options: &UhpOptions) -> GlobalData
    {
        let table_bytes = (options.table_memory * 1e+9) as usize;
        let table = TranspositionTable::new(table_bytes);

        GlobalData {
            args:           SearchArgs::Depth(Depth::new(0)),
            max_depth:      AtomicU64::new(0),
            options:        options.clone(),
            start_time:     Instant::now(),
            stopped:        AtomicBool::new(false),
            transpositions: table,
        }
    }

    /// Sets up the global state to be ready for a search.
    ///
    /// Sets up the manager with the given search args.
    pub fn prepare(&mut self, args: SearchArgs)
    {
        self.args = args;
        self.start_time = Instant::now();
        self.stopped.store(false, Ordering::SeqCst);
        self.transpositions.increment();
    }

    /// Determines if the search should end. If so, it sets the stopped flag as well.
    pub fn should_stop(&self) -> bool
    {
        self.stopped.load(Ordering::SeqCst)
    }

    /// Signals that the engine is out of time.
    pub fn signal(&self)
    {
        self.stopped.store(true, Ordering::SeqCst);
    }
}

#[derive(Clone, Debug)]
/// Contains the thread data per thread, and a view into the global data.
pub struct ThreadData
{
    pub id:         usize,
    pub board:      Board,
    pub variation:  Variation,
    pub target:     i32,
    pub leaf_count: u64,
    pub stem_count: u64,
    pub best_move:  Option<Move>,
    pub cache:      Cache<(ZobristHash, Move), Board>,
}

impl ThreadData
{
    /// Creates a new thread data instance.
    pub fn new(options: UhpOptions, board: &Board) -> ThreadData
    {
        let entry_size = std::mem::size_of::<Board>();
        let threads: usize = std::thread::available_parallelism().map(|nzu| nzu.into()).unwrap_or(0);
        let cap = options.cache_memory * 1e+9 / (threads as f64) / (entry_size as f64);

        ThreadData {
            id:         0,
            board:      board.clone(),
            variation:  Variation::default(),
            target:     0,
            leaf_count: 0,
            stem_count: 0,
            best_move:  None,
            cache:      Cache::new(cap.floor() as u64),
        }
    }

    /// Plays a move but leverages the cache.
    pub fn play(&mut self, mv: &Move)
    {
        self.board.play_unchecked(mv);
    }

    /// Sets up the thread data for the upcoming search.
    pub fn prepare(&mut self)
    {
        self.variation = Variation::default();
        self.target = 0;
        self.leaf_count = 0;
        self.stem_count = 0;
        self.best_move = None;
    }
}
