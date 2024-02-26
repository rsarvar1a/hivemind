use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Represents the current state of the game.
pub enum GameState
{
    NotStarted,
    InProgress,
    Draw,
    WhiteWins,
    BlackWins,
}

impl std::fmt::Display for GameState
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let repr = match self
        {
            | Self::NotStarted => "NotStarted",
            | Self::InProgress => "InProgress",
            | Self::Draw => "Draw",
            | Self::WhiteWins => "WhiteWins",
            | Self::BlackWins => "BlackWins",
        };
        write!(f, "{repr}")
    }
}

impl FromStr for GameState
{
    type Err = Error;
    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err>
    {
        match s
        {
            | "NotStarted" => Ok(Self::NotStarted),
            | "InProgress" => Ok(Self::InProgress),
            | "Draw" => Ok(Self::Draw),
            | "WhiteWins" => Ok(Self::WhiteWins),
            | "BlackWins" => Ok(Self::BlackWins),
            | _ => Err(Error::for_parse::<Self>(s.into())),
        }
    }
}
