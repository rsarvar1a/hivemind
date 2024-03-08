use std::{sync::atomic::Ordering, thread};

use crate::prelude::*;

use itertools::Itertools;
use rand::{thread_rng, seq::SliceRandom};

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
        if board.turn() < 4
        {
            self.sane_opening(board)
        }
        else 
        {
            let moves = super::BasicMoveGenerator::new(board, true).collect::<Vec<Move>>();
            
            if moves.len() == 1
            {
                moves[0]
            }
            else
            {
                self.search(board, args)
            }
        }
    }

    fn generate_moves(board: &Board) -> Self::Generator
    {
        PrioritizingMoveGenerator::new(board, false)
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
    /// Gets the best thread by the score of its variation.
    fn best_thread(&self) -> &ThreadData
    {
        self.thread_data.iter().max_by_key(|t| t.variation.score).unwrap()
    }

    /// Creates the thread data on this evaluator.
    fn create_thread_data<'a>(&mut self, board: &Board)
    {
        let num_threads = std::thread::available_parallelism().map(|nzu| nzu.into()).unwrap_or(1);
        let mut template = ThreadData::new(self.global_data.options.clone(), board);
        self.thread_data.clear();

        for thread_id in 0..num_threads
        {
            template.id = thread_id;
            self.thread_data.push(template.clone());
        }
    }

    /// Returns a sane opening, which is effectively just any opening that does not start with an Ant or Spider.
    fn sane_opening(&self, board: &Board) -> Move
    {
        let turn = board.turn();
        let mut okay_openers = vec![Bug::Beetle, Bug::Grasshopper, Bug::Ladybug, Bug::Pillbug];
        let mut moves = super::BasicMoveGenerator::new(board, true).collect::<Vec<Move>>();
        moves.shuffle(&mut thread_rng());

        if turn < 4
        {
            if (2..=3).contains(& turn)
            {
                // We'll consider early queens and mosquitos as well, since queens could come in early on pillbug games,
                // and mosquitos could potentially function well in pillbug or ladybug openers..
                okay_openers.extend([Bug::Mosquito, Bug::Queen]);
            }

            for mv in moves
            {
                let Move::Place(piece, _) = mv
                else 
                {
                    continue;
                };

                if okay_openers.contains(&piece.kind)
                {
                    return mv;
                }
            }
        }

        unreachable!()
    }

    /// Searches a gamestate for the best continuation.
    fn search(&mut self, board: &Board, args: SearchArgs) -> Move
    {
        self.create_thread_data(board);
        self.setup_data(args);

        thread::scope(|s| {
            
            let global_data = &self.global_data;

            // Our worker threads.
            for mut thread_data in &mut self.thread_data
            {
                s.spawn(move || {
                    Self::iterative_search(global_data, & mut thread_data);
                });
            }

            // Our timer thread.
            if let SearchArgs::Time(duration) = global_data.args
            {
                s.spawn(move || {
                    std::thread::sleep(duration);
                    global_data.signal();
                });
            }
        });

        let mut board = board.clone();
        let best_thread = self.best_thread();
        let variation = best_thread.variation.clone();

        let entry = self.global_data.transpositions.load(board.zobrist()).unwrap();
        let mv = Option::<Move>::from(entry.mv).unwrap_or(variation.moves[0].mv);

        let p = board.to_move();
        board.play(&mv).expect("illegal move");

        let e = -Self::evaluate_board(&board);
        let s = variation.score;

        let lct = self.thread_data.iter().map(|t| t.leaf_count).sum::<u64>();
        let sct = self.thread_data.iter().map(|t| t.stem_count).sum::<u64>();
        let elapsed = self.global_data.start_time.elapsed();
        let el = elapsed.as_secs_f64().round();
        let d = self.global_data.max_depth.load(Ordering::SeqCst);
        let lr = (lct as f64 / el).round() as i32;
        let sr = (sct as f64 / el).round() as i32;
        let el = el as i32;

        let ms = format!("{}", mv);
        let is_variation = best_thread.best_move.is_none();
        let pv = variation.moves.iter().map(|mv| format!("{}", mv.mv)).join(";");
        let pv = if pv.as_str() == "" { "none" } else { pv.as_str() };

        log::info!(r"

 ════════════════════════════════ found move '{ms: ^8}' ═══════════════════════════════
╒═════════╤═════════╤══════════╤═════════╤═══════════╤══════════╤═══════════╤══════════╕
|  score  |   eval  | time (s) |  depth  |   stems   |    /s    |   leaves  |    /s    |
╞═════════╪═════════╪══════════╪═════════╪═══════════╪══════════╪═══════════╪══════════╡
| {s: >7} | {e: >7} | {el: >8} | {d: >7} | {sct: >9} | {sr: >8} | {lct: >9} | {lr: >8} |
╘═════════╧═════════╧══════════╧═════════╧═══════════╧══════════╧═══════════╧══════════╛

player to move: {p}
principal variation: {pv}
variation move? {is_variation}

");

        mv
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
    pub fn new(board: &Board, standard_position: bool) -> Self
    {
        let mut moves = board.generate_moves(standard_position);

        if moves.is_empty()
        {
            moves.push(Move::Pass);
        }

        PrioritizingMoveGenerator { moves, index: 0 }
    }
}
