use super::consts::*;
use crate::prelude::*;

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A direction on a hexagonal grid.
pub enum Direction
{
    East      = 1u16,
    Southeast = ROWS + 1,
    Southwest = ROWS,
    West      = MASK & 1u16.wrapping_neg(),
    Northwest = MASK & (ROWS + 1).wrapping_neg(),
    Northeast = MASK & ROWS.wrapping_neg(),
}

impl Direction
{
    #[inline]
    /// Returns a list of all directions in clockwise order.
    pub const fn all() -> [Direction; 6]
    {
        [Self::East, Self::Southeast, Self::Southwest, Self::West, Self::Northwest, Self::Northeast]
    }

    /// Returns the direction counterclockwise to this one.
    pub fn counterclockwise(&self) -> Direction
    {
        match self
        {
            | Self::East => Self::Northeast,
            | Self::Northeast => Self::Northwest,
            | Self::Northwest => Self::West,
            | Self::West => Self::Southwest,
            | Self::Southwest => Self::Southeast,
            | Self::Southeast => Self::East,
        }
    }

    /// Returns the direction clockwise of this one.
    pub fn clockwise(&self) -> Direction
    {
        match self
        {
            | Self::East => Self::Southeast,
            | Self::Southeast => Self::Southwest,
            | Self::Southwest => Self::West,
            | Self::West => Self::Northwest,
            | Self::Northwest => Self::Northeast,
            | Self::Northeast => Self::East,
        }
    }

    /// Returns the inverse of this direction.
    pub fn inverse(&self) -> Direction
    {
        match self
        {
            | Self::East => Self::West,
            | Self::Southeast => Self::Northwest,
            | Self::Southwest => Self::Northeast,
            | Self::West => Self::East,
            | Self::Northwest => Self::Southeast,
            | Self::Northeast => Self::Southwest,
        }
    }

    /// Determines if this is a west direction (as opposed to an east direction).
    pub fn is_west(&self) -> bool
    {
        matches!(self, Self::West | Self::Northwest | Self::Southwest)
    }

    /// Returns the name of this direction.
    pub fn long(&self) -> &'static str
    {
        match self
        {
            | Self::East => "east",
            | Self::Southeast => "southeast",
            | Self::Southwest => "southwest",
            | Self::West => "west",
            | Self::Northwest => "northwest",
            | Self::Northeast => "northeast",
        }
    }

    /// Attempts to parse this direction.
    pub fn parse(s: &str, on_left: bool) -> Result<Direction>
    {
        match s
        {
            | "-" => Ok(if on_left { Self::West } else { Self::East }),
            | "/" => Ok(if on_left { Self::Southwest } else { Self::Northeast }),
            | "\\" => Ok(if on_left { Self::Northwest } else { Self::Southeast }),
            | _ => Err(Error::for_parse::<Self>(s.into())),
        }
    }

    #[inline]
    /// If the two hexes are neighbours, returns the direction to the target hex.
    pub fn to(from: Hex, to: Hex) -> Option<Direction>
    {
        Direction::all().into_iter().find(|direction| (from + *direction) == to)
    }
}

impl std::fmt::Display for Direction
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let repr = match self
        {
            | Self::East | Self::West => "-",
            | Self::Southeast | Self::Northwest => "\\",
            | Self::Southwest | Self::Northeast => "/",
        };

        write!(f, "{repr}")
    }
}
