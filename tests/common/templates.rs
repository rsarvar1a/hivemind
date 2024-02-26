use hivemind::prelude::*;

/// Runs a game, and checks for both gamestring validity and state mismatch.
pub fn run_game(raw_string: &'static str)
{
    // Ensure the game string is valid.

    let possibly_game_string: Result<GameString> = raw_string.parse::<GameString>();
    assert!(possibly_game_string.is_ok(), "\tdue to {}", possibly_game_string.unwrap_err());

    // Ensure the current state of the board is as expected.

    let board = Board::from(possibly_game_string.unwrap());
    let (game_type_str, state_str, turn_string_str) = game_string_to_parts(raw_string);

    let game_type: GameTypeString = board.options().expansions.into();
    assert_eq!(game_type.as_ref(), game_type_str);

    let state: GameState = board.state();
    assert_eq!(state.to_string(), state_str);

    let turn: Turn = board.turn().into();
    let turn_string: TurnString = turn.into();
    assert_eq!(turn_string.as_ref(), turn_string_str);
}

/// Splits a gamestring (assumed syntactically valid) into parts.
fn game_string_to_parts(game_string: &'static str) -> (&'static str, &'static str, &'static str)
{
    let parts: Vec<&'static str> = game_string.split(";").collect();
    (parts[0], parts[1], parts[2])
}
