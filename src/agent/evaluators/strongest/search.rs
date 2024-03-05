use super::*;

impl StrongestEvaluator
{
    // Performs alpha-beta search.
    fn alpha_beta(global_data: &GlobalData, thread_data: &mut ThreadData, search_data: AlphaBetaSearchData, variation: &mut Variation) -> i32
    {
        // Check time early.
        if global_data.should_stop()
        {
            return 0;
        }

        let mut data = search_data;

        // Clear the variation here, because we reconstruct it from the best found move if there is one.
        variation.moves.clear();

        // Try the transposition table, and check for a cutoff.
        let key = thread_data.board.zobrist();
        if let Some(entry) = global_data.transpositions.load(key, data.depth.floor() as usize)
        {
            // Only consider better depths; otherwise, we'd rather recompute this position.
            if entry.depth >= data.depth
            {
                match entry.bound
                {
                    | TTBound::Exact => return entry.value,
                    | TTBound::Lower =>
                    {
                        data.a = data.a.max(entry.value);
                    }
                    | TTBound::Upper =>
                    {
                        data.b = data.b.min(entry.value);
                    }
                    | _ =>
                    {}
                };

                // Is there a cutoff?
                if data.a >= data.b
                {
                    return entry.value;
                }
            }
        }

        // If we have a depth constraint, enforce it here by returning the heuristic's evaluation.
        // We always have the trivial heuristic of max depth.
        if data.depth == global_data.args.depth().unwrap_or(Depth::MAX)
        {
            return Self::evaluate_board(global_data, thread_data);
        }

        let (mut best_mv, mut best_score, mut bound) = (MoveToken::default(), -NAN, TTBound::Upper);
        let mut new_variation = Variation::default();
        let next_data = AlphaBetaSearchData {
            a:     -data.b,
            b:     -data.a,
            depth: data.depth + 1,
        };

        // Recurse on the movelist.
        for mv in Self::generate_moves(&thread_data.board)
        {
            if let Err(err) = thread_data.board.play(&mv)
            {
                panic!("{}", err);
            }

            let score = -Self::alpha_beta(global_data, thread_data, next_data, &mut new_variation);
            
            if let Err(err) = thread_data.board.undo(1)
            {
                panic!("{}", err);
            }

            // Failure (beta-cutoff).
            if score >= data.b
            {
                let failed_cutoff = TTEntry {
                    key,
                    score: data.b,
                    depth: data.depth,
                    age: TTAge {
                        bound: TTBound::Lower,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                global_data.transpositions.store(&failed_cutoff, data.depth.floor() as usize);
                return data.b;
            }

            // Found next best move, so keep it.
            if score >= best_score
            {
                best_score = score;
                best_mv = mv.into();

                // Then take the variation from the recursive step and load it into the outvariation.
                variation.load(mv, &new_variation);

                if score >= data.a
                {
                    data.a = score;
                    bound = TTBound::Exact;
                }
            }
        }

        // Don't store nulls (Move::Pass) in the table.
        if best_mv.is_some()
        {
            // Store the best move from above.
            let best_entry = TTEntry {
                key,
                mv: best_mv,
                score: best_score,
                depth: data.depth,
                age: TTAge { bound, ..Default::default() },
                ..Default::default()
            };
            global_data.transpositions.store(&best_entry, data.depth.floor() as usize);
        }

        best_score
    }

    // Performs the aspiration loop until an exact score is found.
    fn aspiration_search(
        global_data: &GlobalData,
        thread_data: &mut ThreadData,
        window_data: &mut AspirationSearchData,
        search_depth: Depth,
    ) -> Option<Depth>
    {
        let mut depth = search_depth;

        loop
        {
            let search_data = AlphaBetaSearchData {
                a: window_data.window.a,
                b: window_data.window.b,
                depth,
            };

            // Determine a score for the position.
            window_data.variation.score = Self::alpha_beta(global_data, thread_data, search_data, &mut window_data.variation);

            if global_data.should_stop()
            {
                return None;
            }

            // Failed, so we need to adjust the window and try again.
            if window_data.window.a != -INF && window_data.variation.score <= window_data.window.a
            {
                window_data.window.move_down(window_data.variation.score, depth);
                thread_data.prev();
                continue;
            }

            // The line is either fail-high or correct, so keep it!
            thread_data.next(&window_data.variation);

            // Caused a cut-off, so we need to adjust the window and try again.
            if window_data.window.b != INF && window_data.variation.score >= window_data.window.b
            {
                window_data.window.move_up(window_data.variation.score, depth);
                if window_data.variation.score.abs() < MINIMUM_WIN
                {
                    let min = (search_depth / 2).max(Depth::PLY);
                    depth = (depth - 1).max(min);
                }
                continue;
            }

            let score = window_data.variation.score;
            window_data.average_score = if window_data.average_score == NAN
            {
                score
            }
            else
            {
                (2 * score + window_data.average_score) / 3
            };

            return if global_data.stopped.load(Ordering::SeqCst) { None } else { Some(depth) };
        }
    }

    /// Performs the main iterative deepening loop.
    pub(super) fn iterative_search(global_data: &GlobalData, thread_data: &mut ThreadData, is_main: bool)
    {
        // We vary the starting depth of each thread a bit, so that we can cover different portions of the
        // game tree. Some work done early by deeper threads prepares the transposition table for the threads
        // that start closer to the real position.
        const DEPTH_VARIANCE_BY_THREAD: i32 = 8;

        let search_range = Depth::new(1) + thread_data.id as i32 % DEPTH_VARIANCE_BY_THREAD..=global_data.args.depth().unwrap_or(Depth::MAX);

        // At each depth, we try to narrow the window if possible, but if the search fails, we are forced to
        // undergo a huge (costly) search on the next iteration by setting the window size to be unbounded.
        let mut window_data = AspirationSearchData::default();

        for search_depth in search_range
        {
            thread_data.depth = search_depth;

            // If we ran out of time, we should quit early. Non-main threads will follow at the end of the depth.
            if is_main && global_data.should_stop()
            {
                break;
            }

            // The aspiration main loop might choose a different depth.
            let Some(depth) = Self::aspiration_search(global_data, thread_data, &mut window_data, search_depth)
            else
            {
                break;
            };

            if depth > AspirationWindow::MIN_DEPTH
            {
                window_data.window = AspirationWindow::at(window_data.average_score, depth);
            }
            else
            {
                window_data.window = AspirationWindow::default();
            }

            // For non-main threads, this might be a good place to stop.
            if global_data.should_stop()
            {
                break;
            }
        }

        // If we managed to search to the desired depth, we'll have to stop manually.
        if is_main
        {
            global_data.stopped.store(true, Ordering::SeqCst);
        }
    }
}

#[derive(Clone, Debug)]
/// Data we need in the aspiration loop.
struct AspirationSearchData
{
    pub average_score: i32,
    pub variation:     Variation,
    pub window:        AspirationWindow,
}

impl Default for AspirationSearchData
{
    fn default() -> Self
    {
        AspirationSearchData {
            average_score: scalars::NAN,
            variation:     Variation::default(),
            window:        AspirationWindow::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
/// An aspiration window for narrower search cutoffs.
struct AspirationWindow
{
    pub a_fails: i32,
    pub a:       i32,
    pub b_fails: i32,
    pub b:       i32,
    pub mid:     i32,
}

impl Default for AspirationWindow
{
    /// Returns the unbounded aspiration window.
    fn default() -> Self
    {
        AspirationWindow {
            a_fails: 0,
            a:       -INF,
            b_fails: 0,
            b:       INF,
            mid:     0,
        }
    }
}

impl AspirationWindow
{
    const ASPIRATION_WINDOW: i32 = 6;

    /// The minimum depth of an aspiration window.
    pub const MIN_DEPTH: Depth = Depth::new(Self::ASPIRATION_WINDOW - 1);

    /// Returns a window centred around the given score.
    ///
    /// If that score is a win, then the window is unbounded, because for a majority of the search,
    /// it is unlikely that a found win is forced - thus a narrow search would fail anyways.
    pub fn at(score: i32, depth: Depth) -> Self
    {
        if score >= MINIMUM_WIN
        {
            AspirationWindow {
                mid: score,
                ..Default::default()
            }
        }
        else
        {
            let discriminant = Self::discriminant(depth);
            AspirationWindow {
                a: score - discriminant,
                b: score + discriminant,
                mid: score,
                ..Default::default()
            }
        }
    }

    fn discriminant(depth: Depth) -> i32
    {
        (Self::ASPIRATION_WINDOW + (50 / depth.floor() - (Self::ASPIRATION_WINDOW / 2))).max(10)
    }

    /// Widens the window down, i.e., on a fail-high.
    pub fn move_down(&mut self, score: i32, depth: Depth)
    {
        self.mid = score;

        let diff = Self::discriminant(depth) << (self.a_fails + 1);
        if diff > Self::ASPIRATION_WINDOW.pow(4)
        {
            self.a = -INF;
            return;
        }

        self.b = (self.a + self.b) / 2;
        self.a = self.mid - diff;
        self.a_fails += 1;
    }

    /// Widens the window up, i.e., on a success.
    pub fn move_up(&mut self, score: i32, depth: Depth)
    {
        self.mid = score;

        let diff = Self::discriminant(depth) << (self.b_fails + 1);
        if diff > Self::ASPIRATION_WINDOW.pow(4)
        {
            self.b = INF;
            return;
        }

        self.b = self.mid + diff;
        self.b_fails += 1;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AlphaBetaSearchData
{
    pub a:     i32,
    pub b:     i32,
    pub depth: Depth,
}
