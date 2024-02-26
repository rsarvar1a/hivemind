use multiset::HashMultiSet;
use regex::Regex;

use crate::prelude::*;

#[derive(Clone, Debug)]
/// Represents the type of the game in terms of its enabled expansions.
pub struct GameTypeString(pub(in crate::prelude::notation) String);

impl std::fmt::Display for GameTypeString
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

impl FromStr for GameTypeString
{
    type Err = Error;
    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err>
    {
        let re = Regex::new(r"^Base(\+(?<expansions>[LMP]{1, 3}))?$").unwrap();
        let Some(caps) = re.captures(s)
        else
        {
            return Err(Error::for_parse::<Self>(s.into()));
        };

        if let Some(exps) = caps.name("expansions").map(|m| m.as_str().chars().collect::<HashMultiSet<_>>())
        {
            if ['L', 'M', 'P'].iter().any(|ch| exps.count_of(ch) > 1)
            {
                let err_msg = "should contain at most 1 of each expansion bug (L, M, or P)".into();
                let expansion_err = Error::new(Kind::ParseError, err_msg);
                return Err(expansion_err.chain_parse::<Self>(s.into()));
            }
        }

        Ok(GameTypeString(s.into()))
    }
}

impl AsRef<str> for GameTypeString
{
    fn as_ref(&self) -> &str
    {
        self.0.as_str()
    }
}

impl From<ExpansionOptions> for GameTypeString
{
    fn from(value: ExpansionOptions) -> Self
    {
        let plus = if value.ladybug || value.mosquito || value.pillbug { "+" } else { "" };
        let l = if value.ladybug { "L" } else { "" };
        let m = if value.mosquito { "M" } else { "" };
        let p = if value.pillbug { "P" } else { "" };

        GameTypeString(format!("Base{}{}{}{}", plus, l, m, p))
    }
}

impl From<GameTypeString> for ExpansionOptions
{
    fn from(value: GameTypeString) -> ExpansionOptions
    {
        ExpansionOptions {
            ladybug:  value.0.contains('L'),
            mosquito: value.0.contains('M'),
            pillbug:  value.0.contains('P'),
        }
    }
}

#[derive(Clone, Debug)]
/// Represents a game, including its type and possibly its position.
pub struct GameString
{
    game_type: GameTypeString,
    state:     GameState,
    turn:      TurnString,
    moves:     Vec<MoveString>,
}

impl std::fmt::Display for GameString
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "{};{};{}{}{}",
            self.game_type,
            self.state,
            self.turn,
            if self.moves.is_empty() { "" } else { ";" },
            self.moves.iter().map(|mv| mv.as_ref()).collect::<Vec<_>>().join(";")
        )
    }
}

impl FromStr for GameString
{
    type Err = Error;
    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err>
    {
        // Parse out all of the components.

        let pattern = r"^(?<type>Base(\+[LMP]{1, 3})?);(?<state>[A-Za-z]+);(?<turn>(White|Black)\[[0-9]+\])(?<moves>(;[a-zA-Z1-3\s/\\-]+)*)$";
        let re = Regex::new(pattern).unwrap();

        let Some(caps) = re.captures(s)
        else
        {
            log::trace!("Didn't match regex.");
            return Err(Error::for_parse::<Self>(s.into()));
        };

        let game_type = caps["type"].parse::<GameTypeString>();
        let state = caps["state"].parse::<GameState>();
        let turn = caps["turn"].parse::<TurnString>();

        let Ok(game_type) = game_type
        else
        {
            let err = game_type.err().unwrap();
            return Err(err.chain_parse::<Self>(s.into()));
        };

        let Ok(state) = state
        else
        {
            let err = state.err().unwrap();
            return Err(err.chain_parse::<Self>(s.into()));
        };

        let Ok(turn) = turn
        else
        {
            let err = turn.err().unwrap();
            return Err(err.chain_parse::<Self>(s.into()));
        };

        // Use the above facts to ensure the move list is valid.

        let moves = caps["moves"]
            .to_owned()
            .split_terminator(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<MoveString>())
            .collect::<Result<Vec<MoveString>>>()?;

        let options = Options {
            expansions: game_type.clone().into(),
            ..Default::default()
        };

        let mut board = Board::new(options);

        if let Err(err) = moves
            .iter()
            .map(|mv| {
                let real_move = Move::from(mv, &board)?;
                board
                    .play(&real_move)
                    .map_err(|err| err.chain_parse::<MoveString>(mv.as_ref().to_owned()))?;
                log::trace!("{}\n{:#?}", GameString::from(&board), &board);
                Ok::<(), Error>(())
            })
            .try_collect::<Vec<_>>()
        {
            return Err(err.chain_parse::<Self>(s.into()));
        }

        // Also check that the supplied turn number and gamestate are correct.

        let calculated_state = board.state();
        if state != calculated_state
        {
            let state_mismatch_err = Error::mismatch::<GameState>(state, calculated_state);
            return Err(state_mismatch_err.chain_parse::<Self>(s.into()));
        }

        let calculated_turn: TurnString = Turn::from(board.turn()).into();
        if turn != calculated_turn
        {
            let turn_mismatch_err = Error::mismatch::<TurnString>(turn, calculated_turn);
            return Err(turn_mismatch_err.chain_parse::<Self>(s.into()));
        }

        Ok(GameString {
            game_type,
            state,
            turn,
            moves,
        })
    }
}

impl From<&Board> for GameString
{
    fn from(board: &Board) -> Self
    {
        let game_type: GameTypeString = board.options().expansions.into();
        let state: GameState = board.state();
        let turn: TurnString = Turn::from(board.turn()).into();
        let moves = board
            .history()
            .get_past()
            .iter()
            .rev()
            .map(|mv| mv.mv.into())
            .collect::<Vec<MoveString>>();

        GameString {
            game_type,
            state,
            turn,
            moves,
        }
    }
}

impl From<GameString> for Board
{
    fn from(value: GameString) -> Board
    {
        // Now, we build the board, and apply all of the moves.

        let options = Options {
            expansions: value.game_type.into(),
            ..Default::default()
        };

        let mut board = Board::new(options);

        value.moves.iter().for_each(|mv| {
            let real_move = Move::from(mv, &board).unwrap();
            let move_result = board.play(&real_move);
            if let Err(err) = move_result
            {
                panic!("{}", err);
            }
        });

        board
    }
}
