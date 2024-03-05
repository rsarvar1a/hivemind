use super::*;

const MATERIAL_ADVANTAGE: f64 = 1.5;
const K_MATERIAL: i32 = 1000;

impl StrongestEvaluator
{
    /// Returns a score for the board in white's perspective using some heuristics.
    pub(super) fn evaluate_board(_global_data: &GlobalData, thread_data: &mut ThreadData) -> i32
    {
        match thread_data.board.state()
        {
            | GameState::NotStarted | GameState::Draw => 0,
            | GameState::WhiteWins => MINIMUM_WIN,
            | GameState::BlackWins => -MINIMUM_WIN,
            | _ =>
            {
                let material = Self::evaluate_material(thread_data);
                let to_move = if thread_data.board.to_move() == Player::White { 1 } else { -1 };

                let score = to_move + K_MATERIAL * material;

                score.clamp(-MINIMUM_WIN + 1, MINIMUM_WIN - 1)
            }
        }
    }

    fn evaluate_material(thread_data: &mut ThreadData) -> i32
    {
        // The number of pieces in the game per player.
        let total_per_player = thread_data.board.pouch().extents().iter().sum::<u8>() as i32;

        // Pinned bugs cannot immediately participate in the attack.
        let white_pinned = thread_data.board.pinned_pieces(Player::White).len() as i32;
        let black_pinned = thread_data.board.pinned_pieces(Player::Black).len() as i32;

        // The asset score of each player is the number of bugs available to participate in the attack.
        let white_material = total_per_player - white_pinned;
        let black_material = total_per_player - black_pinned;

        // We assume the position to be white-favoured by some amount proportional to its attacking strength.
        // If there is a significant advantage, boost it.
        let diff = white_material - black_material;
        diff.signum() * (diff.abs() as f64).powf(MATERIAL_ADVANTAGE).floor() as i32
    }
}
