use std::{sync::atomic::Ordering, thread};

use crate::prelude::*;

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
            // We can skip the search entirely with this check.
            // Otherwise, even in DTM-1 positions, it struggles.
            if let Some(mate) = self.mate_in_one(board)
            {
                return mate;
            }

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

    /// Perhaps there's a mate in one here, in which case we should skip discovery.
    fn mate_in_one(&self, board: &Board) -> Option<Move>
    {
        let mut board = board.clone();
        let undo = board.clone();
        let to_move = board.to_move();
        let expect = if to_move == Player::White { GameState::WhiteWins } else { GameState::BlackWins };

        let moves = super::BasicMoveGenerator::new(&board, true).collect::<Vec<Move>>();
        for mv in moves
        {
            board.play_unchecked(&mv);
            if board.state() == expect
            {
                return Some(mv);
            }
            board = undo.clone();
        }
        None
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
            for (index, data) in self.thread_data.iter_mut().enumerate()
            {
                s.spawn(move || {
                    Self::iterative_search(global_data, data, index == 0);
                });
            }
        });

        let leaf_count = self.thread_data.iter().map(|t| t.leaf_count).sum::<u64>();
        let stem_count = self.thread_data.iter().map(|t| t.stem_count).sum::<u64>();
        let time_elapsed = self.global_data.start_time.elapsed();

        let best_thread = self.best_thread();
        let mut movegen = super::BasicMoveGenerator::new(board, true);
        let principal_variation = best_thread.principal_variation();
        let mv = principal_variation.moves.first().copied().unwrap_or(movegen.next().unwrap());
        let best_score = principal_variation.score;

        log::debug!("found {: ^8}: scored {: >6}", mv, best_score);
        log::debug!("took {: >3.1}s and reached depth {}", time_elapsed.as_secs_f64(), self.global_data.max_depth.load(Ordering::SeqCst));
        log::debug!("visited {:09}  stems ({: >6} N/s)", stem_count, (stem_count as f64 / time_elapsed.as_secs_f64()).floor() as u32);
        log::debug!("visited {:09} leaves ({: >6} N/s)", leaf_count, (leaf_count as f64 / time_elapsed.as_secs_f64()).floor() as u32);

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
