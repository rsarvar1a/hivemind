use crate::prelude::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// The players in a game of Hive.
pub enum Player
{
    White = 0,
    Black = 1,
}

impl Player
{
    /// Gets the next player.
    pub fn flip(&self) -> Self
    {
        match self
        {
            | Self::White => Self::Black,
            | Self::Black => Self::White,
        }
    }

    /// Gets the player from its index.
    pub fn new(i: u8) -> Self
    {
        let val = i.clamp(0, 1);
        unsafe { std::mem::transmute::<u8, Player>(val) }
    }

    // Returns the short name for this player, for use in piece and move notation.
    pub fn short(&self) -> &'static str
    {
        match self
        {
            | Self::White => "w",
            | Self::Black => "b",
        }
    }
}

impl std::fmt::Display for Player
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let name = match self
        {
            | Self::White => "White",
            | Self::Black => "Black",
        };
        write!(f, "{name}")
    }
}

impl FromStr for Player
{
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        match s
        {
            | "White" | "w" => Ok(Self::White),
            | "Black" | "b" => Ok(Self::Black),
            | _ => Err(Error::for_parse::<Self>(s.into())),
        }
    }
}
