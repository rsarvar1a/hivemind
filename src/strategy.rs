use std::collections::HashSet;

use minimax::{Evaluation, Evaluator, Game, Winner, BEST_EVAL, WORST_EVAL};

use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Hive;

impl Game for Hive
{
    type S = Board;
    type M = Move;

    fn generate_moves(state: &Self::S, moves: &mut Vec<Self::M>)
    {
        state.generate_moves(moves, true);
    }

    fn get_winner(state: &Self::S) -> Option<minimax::Winner>
    {
        let gamestate = state.state();
        match gamestate
        {
            | GameState::NotStarted | GameState::InProgress => None,
            | GameState::Draw => Some(Winner::Draw),
            | GameState::WhiteWins => Some(match state.to_move()
            {
                | Player::White => Winner::PlayerToMove,
                | _ => Winner::PlayerJustMoved,
            }),
            | GameState::BlackWins => Some(match state.to_move()
            {
                | Player::Black => Winner::PlayerToMove,
                | _ => Winner::PlayerJustMoved,
            }),
        }
    }

    fn apply(state: &mut Self::S, m: Self::M) -> Option<Self::S>
    {
        let mut new_state = state.clone();
        // if let Err(e) = new_state.check(&m)
        // {
        //     let gamestr = GameString::from(&* state);
        //     log::error!("encountered error in minimax:\n{}\n{}\nin state: {:#?}\nposition: {}", e, m, state, gamestr);
        //     panic!();
        // }

        new_state.play_unchecked(&m);
        Some(new_state)
    }

    fn zobrist_hash(_state: &Self::S) -> u64
    {
        _state.zobrist() as u64
    }

    fn null_move(_state: &Self::S) -> Option<Self::M>
    {
        let mut moves = Vec::new();
        _state.generate_moves(&mut moves, false);
        moves.is_empty().then_some(Move::Pass)
    }

    fn notation(_state: &Self::S, _move: Self::M) -> Option<String>
    {
        Some(_move.to_string())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HiveEval;

impl Evaluator for HiveEval
{
    type G = Hive;
    fn evaluate(&self, s: &<Self::G as Game>::S) -> minimax::Evaluation
    {
        evaluate_board(s).clamp(WORST_EVAL as i32, BEST_EVAL as i32) as Evaluation
    }

    fn generate_noisy_moves(&self, _state: &<Self::G as Game>::S, _moves: &mut Vec<<Self::G as Game>::M>)
    {
        _state.generate_tactical_moves(_moves);
    }
}

const ATTACKING_KILLSPOT: f64 = 1.2;
const MINIMUM_OPEN_KILLSPOTS: usize = 2;

const K_DEFENSE: f64 = 40.0;
const K_MOVEABLE: f64 = 2.0;
const K_QUEEN_NEIGHBOURHOOD: f64 = 30.0;
const K_QUEENS: f64 = 1.0;
const K_RESERVE: f64 = 1.0;
const K_STACKING: f64 = 2.0;

const VALUE_ANT: f64 = 7.0;
const VALUE_BEETLE: f64 = 6.0;
const VALUE_GRASSHOPPER: f64 = 3.0;
const VALUE_LADYBUG: f64 = 6.0;
const VALUE_MOSQUITO: f64 = 8.0;
const VALUE_PILLBUG: f64 = 6.0;
const VALUE_QUEEN: f64 = 12.0;
const VALUE_SPIDER: f64 = 2.0;

const MINIMUM_WIN: i32 = BEST_EVAL as i32;

/// Gives a baseline value for a piece. The queen value is HIGH, because it refers to moveable queens.
///
/// Moveable queens are strong because they can totally neutralize an opponent's tempo by escaping an attack.
fn bug_value(bug: Bug) -> f64
{
    match bug
    {
        | Bug::Ant => VALUE_ANT,
        | Bug::Beetle => VALUE_BEETLE,
        | Bug::Grasshopper => VALUE_GRASSHOPPER,
        | Bug::Ladybug => VALUE_LADYBUG,
        | Bug::Mosquito => VALUE_MOSQUITO,
        | Bug::Pillbug => VALUE_PILLBUG,
        | Bug::Queen => VALUE_QUEEN,
        | Bug::Spider => VALUE_SPIDER,
    }
}

/// Returns a score for the board in the moving player's perspective using some heuristics.
pub(super) fn evaluate_board(board: &Board) -> i32
{
    let is_white = if board.to_move() == Player::White { 1 } else { -1 };
    let is_black = -is_white;

    match board.state()
    {
        | GameState::NotStarted | GameState::Draw => 0,
        | GameState::WhiteWins => MINIMUM_WIN * is_white,
        | GameState::BlackWins => MINIMUM_WIN * is_black,
        | _ =>
        {
            let score = material(board) + queens(board) + reserve(board);
            let integer_score = score.floor() as i32;
            integer_score.clamp(-MINIMUM_WIN + 1, MINIMUM_WIN - 1)
        }
    }
}

/// Returns the material advantage in the moving player's perspective, which is roughly the difference in board strength.
fn material(board: &Board) -> f64
{
    let mut score = 0.0;
    let occupied: HashSet<Hex> = board.field().clone().into();

    for hex in occupied
    {
        let piece = board.top(hex).unwrap();

        // Discard pinned bugs.
        if board.is_pinned(&piece)
        {
            continue;
        }

        let mut piece_score = bug_value(piece.kind);
        let stacking = board.stacked(&piece);

        // Special check to rescore a mosquito as its best possible neighbour.
        'mosquito_rescore: {
            if piece.kind == Bug::Mosquito
            {
                if stacking
                {
                    piece_score = bug_value(Bug::Beetle);
                    break 'mosquito_rescore;
                }
                for neighbour in board.pieces_neighbouring(hex)
                {
                    if neighbour.kind == Bug::Queen
                    {
                        continue;
                    }

                    piece_score = piece_score.max(bug_value(neighbour.kind));
                }
            }
        }

        // Heavily reweight a bug that is on a stack, because it is pinning something underneath!
        if stacking
        {
            piece_score *= K_STACKING;
        }

        // Bugs attacking their enemy queen should attack in worst-value-first order, to save high-value pieces for overall pressure in the Hive.
        if let Some(enemy_queen_loc) = board.queen(piece.player.flip())
        {
            if board.field().neighbours(hex).iter().any(|adj| *adj == enemy_queen_loc)
            {
                piece_score = 0.0;
            }
        }

        // Invert it if it does not belong to the current player.
        if piece.player != board.to_move()
        {
            piece_score *= -1.0;
        }

        score += piece_score;
    }

    K_MOVEABLE * score
}

/// Returns a metric calculating the relative safety of the queens. This includes pillbug defense, if possible!
fn queens(board: &Board) -> f64
{
    fn queen_score_for(board: &Board, player: Player) -> f64
    {
        let mut score = 0.0;
        let crawlers = vec![Bug::Ant, Bug::Mosquito, Bug::Pillbug, Bug::Queen, Bug::Spider];

        // Check for the safety of the friendly queen.
        if let Some(queen_hex) = board.queen(player)
        {
            let queen = Piece {
                player,
                kind: Bug::Queen,
                num: 1,
            };

            for neighbour in board.neighbours(queen_hex)
            {
                // If this bug is friendly, we can assume the best about its future moves.
                // For instance, it vacates killspots, or performs good warps.
                if neighbour.player == player
                {
                    // If the bug is a crawler, then it's blocked if we gave it Ant powers
                    // and it still couldn't vacate the hex it's on.
                    let from = board.location(&neighbour).unwrap();
                    let is_blocked = crawlers.contains(&neighbour.kind) && board.is_blocked_crawler(from);

                    // Check if the queen's killspots are filled.
                    // If we can vacate a killspot, it is not that severe.
                    score -= if is_blocked || board.is_pinned(&neighbour)
                    {
                        K_QUEEN_NEIGHBOURHOOD
                    }
                    else
                    {
                        K_QUEEN_NEIGHBOURHOOD / 2.0
                    };

                    // Check if a friendly pillbug or mosquito can warp the queen to safety.
                    if board.can_throw_another(&neighbour) && !board.is_pinned(&queen)
                    {
                        let mut escapes: Vec<usize> = Vec::new();

                        let from = queen_hex;
                        let intermediate = board.location(&neighbour).unwrap();
                        for to in hex::neighbours(intermediate)
                        {
                            let open_killspots = 6 - board.field().neighbours(to).len();
                            if !board.occupied(to) && board.check_throw_via(from, neighbour, to).is_ok()
                            {
                                escapes.push(open_killspots);
                            }
                        }

                        // If we have a suitable defense, reward even further.
                        let best = escapes.into_iter().max().unwrap_or(0);
                        if best > MINIMUM_OPEN_KILLSPOTS
                        {
                            score += K_DEFENSE;
                        }
                    }
                }
                // Otherwise, we can assume the bugs will not vacate killspots except for in exceptional tempo cases.
                else
                {
                    // There is a heavy penalty to having a killspot filled.
                    score -= K_QUEEN_NEIGHBOURHOOD * ATTACKING_KILLSPOT;

                    // Check how much damage an opponent pillbug could do to the queen's position.
                    if board.can_throw_another(&neighbour) && !board.is_pinned(&queen)
                    {
                        let mut escapes: Vec<usize> = Vec::new();

                        let from = queen_hex;
                        let intermediate = board.location(&neighbour).unwrap();
                        for to in hex::neighbours(intermediate)
                        {
                            let open_killspots = 6 - board.field().neighbours(to).len();
                            if !board.occupied(to) && board.check_throw_via(from, neighbour, to).is_ok()
                            {
                                escapes.push(open_killspots);
                            }
                        }

                        let best = escapes.into_iter().min().unwrap_or(6);
                        if best <= MINIMUM_OPEN_KILLSPOTS
                        {
                            score -= K_QUEEN_NEIGHBOURHOOD;
                        }
                    }
                }
            }

            // Finally, if the pillbug itself is not placed, and we could direct-drop it next to the queen, add a contingency reward.
            let pillbug = Piece {
                player,
                kind: Bug::Pillbug,
                num: 1,
            };

            if board.location(&pillbug).is_none()
            {
                for neighbour in hex::neighbours(queen_hex)
                {
                    // If we found an empty neighbour with no unfriendly neighbours, we succeeded.
                    if !board.occupied(neighbour) && !board.neighbours(neighbour).iter().any(|piece| piece.player != player)
                    {
                        score += K_DEFENSE / 2.0;
                        break;
                    }
                }
            }
        }

        score
    }

    let to_move = board.to_move();
    let score = queen_score_for(board, to_move) - queen_score_for(board, to_move.flip());
    K_QUEENS * score
}

/// Returns the in-hand advantage in the moving player's perspective.
fn reserve(board: &Board) -> f64
{
    fn reserve_for(board: &Board, player: Player) -> f64
    {
        let mut score = 0.0;
        for bug in Bug::all().iter()
        {
            let remaining = board.pouch().peek(player, *bug).unwrap_or(0);
            score += bug_value(*bug) + remaining as f64;
        }
        score
    }

    let to_move = board.to_move();
    let score = reserve_for(board, to_move) - reserve_for(board, to_move.flip());
    K_RESERVE * score
}
