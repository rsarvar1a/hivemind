use regex::Regex;

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Represents a valid (checked) turn string.
///
/// A turn string is of the form `Player[Turn]`; for example, `White[1]`.
pub struct TurnString(String);

impl std::fmt::Display for TurnString
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TurnString
{
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        let turn = s.parse::<Turn>()?;
        Ok(turn.into())
    }
}

impl AsRef<str> for TurnString
{
    fn as_ref(&self) -> &str
    {
        self.0.as_str()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Represents a plied turn in HIve.
///
/// The turn number in a Turn only increments once both players have made a move on the previous number.
pub struct Turn
{
    pub player: Player,
    pub turn:   u8,
}

impl From<u8> for Turn
{
    fn from(value: u8) -> Self
    {
        let player = Player::new(value & 0x1);
        let turn = (value >> 1) + 1;
        Turn { player, turn }
    }
}

impl From<Turn> for u8
{
    fn from(value: Turn) -> u8
    {
        (value.turn << 1) + (value.player as u8)
    }
}

impl From<Turn> for TurnString
{
    fn from(value: Turn) -> TurnString
    {
        TurnString(format!("{}[{}]", value.player, value.turn))
    }
}

impl From<TurnString> for Turn
{
    fn from(value: TurnString) -> Self
    {
        let player_str: String = value.0[0..5].into();
        let player = player_str.parse::<Player>().unwrap();

        let n = value.0.len();
        let turn_str: String = value.0[6..n].into();
        let turn = turn_str.parse::<u8>().unwrap();

        Turn { player, turn }
    }
}

impl FromStr for Turn
{
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        let re = Regex::new(r"^(?<player>White|Black)\[(?<turn>[0-9]+)\]$").unwrap();

        let Some(caps) = re.captures(s)
        else
        {
            return Err(Error::for_parse::<Self>(s.into()));
        };

        let player = caps["player"].parse::<Player>();
        let turn = caps["turn"].parse::<u8>();

        let Ok(player) = player
        else
        {
            let err = player.err().unwrap();
            return Err(err.chain_parse::<Self>(s.into()));
        };

        let Ok(turn) = turn
        else
        {
            let err = Error::for_parse::<u8>(caps["turn"].into());
            return Err(err.chain_parse::<Self>(s.into()));
        };

        if turn == 0
        {
            let turn_error = Error::new(Kind::LogicError, "Turn number cannot be 0.".into());
            return Err(turn_error.chain_parse::<Self>(s.into()));
        }

        Ok(Turn { player, turn })
    }
}
