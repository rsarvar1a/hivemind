use super::*;

impl StrongestEvaluator
{
    // Performs alpha-beta search.
    fn alpha_beta(global_data: &GlobalData, thread_data: &mut ThreadData, search_data: ABData, prev: Option<Move>) -> Option<i32>
    {
        // Check time early.
        if global_data.should_stop()
        {
            return None;
        }

        let mut data = search_data;
        thread_data.leaf_count += 1;

        // If we are in a terminal state, we should also return immediately.
        if matches!(thread_data.board.state(), GameState::WhiteWins | GameState::BlackWins | GameState::Draw)
        {
            return Some(Self::evaluate_board(&thread_data.board));
        }

        // If we have a depth constraint, find extensions using quiescence search, and return the static evaluation at the q-root.
        if data.depth <= Depth::NIL
        {
            const QUIESCENCE_DEPTH: Depth = Depth::new(2);

            let q_data = ABData {
                a:     data.a,
                b:     data.b,
                depth: QUIESCENCE_DEPTH,
            };

            return Self::quiescence(global_data, thread_data, q_data);
        }

        let pre_alpha = data.a;
        let mut candidate = None;
        let board = thread_data.board.clone();

        // We might have a good move in the table.
        if let Some(score) = global_data
            .transpositions
            .check(board.zobrist(), data.depth, &mut candidate, &mut data.a, &mut data.b)
        {
            return Some(score);
        }

        // Null move observation holds?
        if Self::bugzwang(global_data, thread_data, data.clone())? >= data.b
        {
            return Some(data.b);
        }

        let mut moves = super::PrioritizingMoveGenerator::new(&board, false).collect::<Vec<_>>();

        // If we are stunlocked, we're probably dead.
        if moves[0] == Move::Pass
        {
            return Some(MINIMUM_LOSS);
        }
        // Singular extensions.
        else if moves.len() == 1
        {
            data.depth += Depth::PLY;
        }

        // Try our table move first.
        if let Some(table_move) = candidate
        {
            for i in 0..moves.len()
            {
                if moves[i] == table_move
                {
                    moves[0..i + 1].rotate_right(1);
                    break;
                }
            }
        }

        let mut best_score = MINIMUM_LOSS;
        let mut best_mv = moves[0];
        let mut null_window = false;

        for mv in moves.iter()
        {
            thread_data.play(mv);

            let v = if null_window
            {
                let null_data = ABData {
                    a:     -data.a - 1,
                    b:     -data.a,
                    depth: data.depth - Depth::PLY,
                };

                let attempt = -Self::alpha_beta(global_data, thread_data, null_data, Some(*mv))?;

                if (data.a + 1..=data.b - 1).contains(&attempt)
                {
                    let next = ABData {
                        a: -data.b,
                        b: -attempt,
                        depth: data.depth - Depth::PLY
                    };

                    -Self::alpha_beta(global_data, thread_data, next, Some(*mv))?
                }
                else
                {
                    attempt
                }
            }
            else
            {
                let next = ABData {
                    a:     -data.b,
                    b:     -data.a,
                    depth: data.depth - Depth::PLY,
                };

                -Self::alpha_beta(global_data, thread_data, next, Some(*mv))?
            };

            thread_data.board = board.clone();

            if v > best_score 
            {
                best_score = v;
                best_mv = *mv;
            }

            if v > data.a
            {
                data.a = v;
                null_window = true;
            }

            if data.a >= data.b
            {
                let _p = prev;
                break;
            }
        }

        thread_data.leaf_count -= 1;
        thread_data.stem_count += 1;

        // Something is wrong if we're getting here with passes.
        assert!(best_mv != Move::Pass);

        let entry = TTEntry {
            key: thread_data.board.zobrist(),
            mv: best_mv.into(),
            depth: data.depth.clone(),
            score: best_score,
            age: TTAge::compute(best_score, pre_alpha, data.b)
        };

        global_data.transpositions.store(&entry);
        Some(scores::normalize(best_score))
    }

    // Performs the aspiration loop until an exact score is found.
    fn aspiration_search(global_data: &GlobalData, thread_data: &mut ThreadData, search_depth: Depth, window: i32) -> Option<()>
    {
        if search_depth < Depth::new(2)
        {
            Some(())
        }
        else
        {
            let target_score = thread_data.target;
            let search_data = ABData {
                a:     target_score.saturating_sub(window).max(MINIMUM_LOSS),
                b:     target_score.saturating_add(window).min(MINIMUM_WIN),
                depth: search_depth,
            };

            Self::alpha_beta(global_data, thread_data, search_data, None)?;
            Some(())
        }
    }

    // Looks for the null move observation.
    fn bugzwang(global_data: &GlobalData, thread_data: &mut ThreadData, search_data: ABData) -> Option<i32>
    {
        const DEPTH_REDUCTION: Depth = Depth::new(2);
        
        let data = search_data;
        let board = thread_data.board.clone();
        let movegen = super::PrioritizingMoveGenerator::new(&board, false).collect::<Vec<_>>();

        if movegen.contains(&Move::Pass)
        {
            if data.depth > DEPTH_REDUCTION && Self::evaluate_board(&board) >= data.b
            {
                let next_data = ABData {
                    a: -data.b,
                    b: -data.b + 1,
                    depth: data.depth - DEPTH_REDUCTION
                };

                thread_data.play(&Move::Pass);
                let v = -Self::alpha_beta(global_data, thread_data, next_data, None)?;
                thread_data.board = board.clone();

                if v >= data.b
                {
                    return Some(v);
                }
            }
        }

        Some(MINIMUM_LOSS)
    }

    /// Performs the main iterative deepening loop.
    pub(super) fn iterative_search(global_data: &GlobalData, thread_data: &mut ThreadData)
    {
        // We vary the starting depth of each thread a bit, so that we can cover different portions of the
        // game tree. Some work done early by deeper threads prepares the transposition table for the threads
        // that start closer to the real position.
        const DEPTH_VARIANCE_BY_THREAD: i32 = 2;

        let search_range = Depth::new(1) + thread_data.id as i32 % DEPTH_VARIANCE_BY_THREAD..=global_data.args.depth();

        // Get the root moves so we can reorder them.
        let board = thread_data.board.clone();
        let mut moves = super::PrioritizingMoveGenerator::new(&board, true)
            .map(|mv| ScoredMove { mv, score: 0 })
            .collect::<Vec<_>>();

        for search_depth in search_range
        {
            // Try our window search first.
            if Self::aspiration_search(global_data, thread_data, search_depth, ABData::ASPIRATION_WINDOW).is_none()
            {
                break;
            }

            // Conduct a search from the root, reordering moves in greatest-score-order while doing so.
            if Self::reordering_search(global_data, thread_data, &mut moves, search_depth).is_none()
            {
                break;
            };

            // Update the max depth window seen.
            if search_depth.floor() as u64 > global_data.max_depth.load(Ordering::SeqCst)
            {
                global_data.max_depth.store(search_depth.floor() as u64, Ordering::SeqCst);
            }

            // Check the root result. If it's a win score, we just abort early.
            let hit = global_data.transpositions.load(board.zobrist()).unwrap();
            
            // Remember this result.
            thread_data.target = hit.score;
            thread_data.best_move = hit.mv.into();

            // Load the principal variation scores from the table.
            global_data
                .transpositions
                .get_principal_variation(&board, &mut thread_data.variation);

            if scores::reconstruct(hit.score).abs() == MINIMUM_WIN
            {
                break;
            }
        }
    }

    // Computes exciting extensions at leaves to ensure we don't miss tactical resolutions due to the horizon effect.
    fn quiescence(global_data: &GlobalData, thread_data: &mut ThreadData, search_data: ABData) -> Option<i32>
    {
        if global_data.should_stop()
        {
            return None;
        }

        let mut data = search_data;

        if data.depth <= Depth::NIL || matches!(thread_data.board.state(), GameState::WhiteWins | GameState::BlackWins | GameState::Draw)
        {
            return Some(Self::evaluate_board(&thread_data.board));
        }
        
        let board = thread_data.board.clone();
        let moves = board.generate_tactical_moves();
        let mut best_score = MINIMUM_LOSS;

        if moves.is_empty()
        {
            return Some(Self::evaluate_board(&board));
        }

        for mv in moves.iter()
        {
            let next_data = ABData {
                a: -data.b,
                b: -data.a,
                depth: data.depth - Depth::PLY
            };

            thread_data.play(mv);
            let v = -Self::quiescence(global_data, thread_data, next_data)?;
            thread_data.board = board.clone();

            best_score = best_score.max(v);
            data.a = data.a.max(v);

            if data.a >= data.b 
            {
                break;
            }
        }

        Some(best_score)
    }

    /// Searches through the moves, reordering them by their evaluation.
    fn reordering_search(global_data: &GlobalData, thread_data: &mut ThreadData, moves: &mut [ScoredMove], depth: Depth) -> Option<()>
    {
        let mut data = ABData {
            a:     MINIMUM_LOSS,
            b:     MINIMUM_WIN,
            depth: depth - Depth::PLY,
        };

        let board = thread_data.board.clone();

        for mv in moves.iter_mut()
        {
            thread_data.play(&mv.mv);
            mv.score = -Self::alpha_beta(global_data, thread_data, data.clone(), Some(mv.mv.clone()))?;
            thread_data.board = board.clone();

            data.a = data.a.max(mv.score);
        }

        // Put the strongest moves at the front.
        moves.sort_by_key(|mv| -mv.score);
        let ScoredMove { mv, score } = moves[0];

        let entry = TTEntry {
            key: thread_data.board.zobrist(),
            mv: mv.into(),
            score,
            depth,
            age: TTAge::compute(score, data.a, data.b),
        };

        global_data.transpositions.store(&entry);
        Some(())
    }
}

#[derive(Clone, Debug)]
/// Data we need in the aspiration loop.
struct ABData
{
    pub a:     i32,
    pub b:     i32,
    pub depth: Depth,
}

impl Default for ABData
{
    fn default() -> Self
    {
        ABData {
            a:     0,
            b:     0,
            depth: Depth::NIL,
        }
    }
}

impl ABData
{
    // The default radius of the aspiration window.
    pub const ASPIRATION_WINDOW: i32 = 50;
}
