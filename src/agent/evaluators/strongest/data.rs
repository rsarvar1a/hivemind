use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
};

use crate::prelude::*;

#[derive(Debug)]
/// Contains any information shared between threads.
pub struct GlobalData
{
    pub args:           SearchArgs,
    pub max_depth:      AtomicU64,
    pub start_time:     Instant,
    pub stopped:        AtomicBool,
    pub transpositions: TranspositionTable,
}

impl GlobalData
{
    /// Creates a new GlobalData with the given options.
    pub fn new(options: &UhpOptions) -> GlobalData
    {
        let bytes = (options.memory * 1e+9) as usize;
        let table = TranspositionTable::new(bytes);

        GlobalData {
            args:           SearchArgs::Depth(Depth::new(0)),
            max_depth:      AtomicU64::new(0),
            start_time:     Instant::now(),
            stopped:        AtomicBool::new(false),
            transpositions: table,
        }
    }

    /// Determines time control (if the search args are set to time).
    pub fn out_of_time(&self) -> bool
    {
        match self.args
        {
            | SearchArgs::Depth(_) => false,
            | SearchArgs::Time(duration) => self.start_time.elapsed() > duration,
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
        if self.stopped.load(Ordering::SeqCst) || self.out_of_time()
        {
            self.stopped.store(true, Ordering::SeqCst);
            true
        }
        else
        {
            false
        }
    }
}

#[derive(Clone, Debug)]
/// Contains the thread data per thread, and a view into the global data.
pub struct ThreadData
{
    pub id:         usize,
    pub board:      Board,
    pub evals:      Vec<i32>,
    pub variations: Vec<Variation>,
    pub depth:      Depth,
    pub finished:   Depth,
}

impl ThreadData
{
    /// Gets the highest depth completed by this thread.
    pub fn depth(&self) -> usize
    {
        self.finished.floor() as usize
    }

    /// Creates a new thread data instance.
    pub fn new(board: &Board) -> ThreadData
    {
        ThreadData {
            id:         0,
            board:      board.clone(),
            evals:      vec![0; MAXIMUM_PLY],
            variations: vec![Variation::default(); MAXIMUM_PLY],
            depth:      Depth::new(0),
            finished:   Depth::new(0),
        }
    }

    pub fn next(&mut self, variation: &Variation)
    {
        self.finished = self.depth;
        let index = self.depth();
        self.variations[index] = variation.clone();
    }

    /// Sets up the thread data for the upcoming search.
    pub fn prepare(&mut self)
    {
        self.depth = Depth::new(0);
        self.finished = Depth::new(0);
        self.variations.fill(Variation::default());
    }

    /// Steps back to the previous variation.
    pub fn prev(&mut self)
    {
        self.finished = self.depth - 1;
    }

    /// Gets the principal variation, which is the best variation found at the highest completed depth.
    pub fn principal_variation(&self) -> &Variation
    {
        &self.variations[self.depth()]
    }
}
