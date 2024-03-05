use std::{sync::atomic::Ordering, thread};

use crate::prelude::*;

mod data;
mod evaluate;
mod search;

use data::*;

#[derive(Debug)]
/// An evaluator based on alpha-beta search with a set of custom heuristics.
pub struct StrongestEvaluator
{
    global_data: GlobalData,
    thread_data: Vec<ThreadData>,
}

impl Evaluator for StrongestEvaluator
{
    type Generator = PrioritizingMoveGenerator;

    fn best_move(&mut self, board: &Board, args: SearchArgs) -> Move
    {
        let moves = super::BasicMoveGenerator::new(board).collect::<Vec<Move>>();

        if moves.len() == 1
        {
            moves[0]
        }
        else
        {
            self.search(board, args)
        }
    }

    fn generate_moves(board: &Board) -> Self::Generator
    {
        PrioritizingMoveGenerator::new(board)
    }

    fn new(options: UhpOptions) -> Self
    {
        let global_data = GlobalData::new(&options);

        StrongestEvaluator {
            global_data,
            thread_data: Vec::new(),
        }
    }
}

impl StrongestEvaluator
{
    /// Gets the thread with the best search performance.
    fn best_thread(&self) -> &ThreadData
    {
        let (mut best, rest) = self.thread_data.split_first().unwrap();
        for this in rest
        {
            let (best_depth, best_score) = (best.depth(), best.principal_variation().score);
            let (this_depth, this_score) = (this.depth(), this.principal_variation().score);
            if ((this_depth == best_depth || this_score > scores::MINIMUM_WIN) && this_score > best_score)
                || (this_depth > best_depth && (this_score > best_score || best_score < scores::MINIMUM_WIN))
            {
                best = this;
            }
        }
        best
    }

    /// Creates the thread data on this evaluator.
    fn create_thread_data<'a>(&mut self, board: &Board)
    {
        let num_threads = std::thread::available_parallelism().map(|nzu| nzu.into()).unwrap_or(1);
        let mut template = ThreadData::new(board);
        self.thread_data.clear();

        for thread_id in 0..num_threads
        {
            template.id = thread_id;
            self.thread_data.push(template.clone());
        }
    }

    /// Searches a gamestate for the best continuation.
    fn search(&mut self, board: &Board, args: SearchArgs) -> Move
    {
        self.create_thread_data(board);
        self.setup_data(args);

        thread::scope(|s| {
            let global_data = &self.global_data;
            for (index, data) in self.thread_data.iter_mut().enumerate()
            {
                s.spawn(move || {
                    Self::iterative_search(global_data, data, index == 0);
                });
            }
        });

        let best_thread = self.best_thread();
        best_thread.principal_variation().moves.first().copied().unwrap_or(Move::Pass)
    }

    /// Sets up the thread data and global data to prepare for a search.
    fn setup_data(&mut self, args: SearchArgs)
    {
        self.global_data.prepare(args);
        for t in self.thread_data.iter_mut()
        {
            t.prepare();
        }
    }
}

/// A move generator that tries to generate better moves first for the purposes of
/// making position evaluation more efficient.
pub struct PrioritizingMoveGenerator
{
    moves: Vec<Move>,
    index: usize,
}

impl Iterator for PrioritizingMoveGenerator
{
    type Item = Move;
    fn next(&mut self) -> Option<Self::Item>
    {
        let next = self.moves.get(self.index);
        self.index += 1;
        next.copied()
    }
}

impl<'a> PrioritizingMoveGenerator
{
    pub fn new(board: &Board) -> Self
    {
        let mut moves = board.generate_moves();

        if moves.is_empty()
        {
            moves.push(Move::Pass);
        }

        PrioritizingMoveGenerator { moves, index: 0 }
    }
}
