use crate::prelude::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// The types of bugs in Hive.
pub enum Bug
{
    Ant         = 0,
    Beetle      = 1,
    Grasshopper = 2,
    Ladybug     = 3,
    Mosquito    = 4,
    Pillbug     = 5,
    Queen       = 6,
    Spider      = 7,
}

impl std::fmt::Display for Bug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let name = match self
        {
            | Self::Ant => "A",
            | Self::Beetle => "B",
            | Self::Grasshopper => "G",
            | Self::Ladybug => "L",
            | Self::Mosquito => "M",
            | Self::Pillbug => "P",
            | Self::Queen => "Q",
            | Self::Spider => "S",
        };
        write!(f, "{name}")
    }
}

impl FromStr for Bug
{
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        match s
        {
            | "A" => Ok(Self::Ant),
            | "B" => Ok(Self::Beetle),
            | "G" => Ok(Self::Grasshopper),
            | "L" => Ok(Self::Ladybug),
            | "M" => Ok(Self::Mosquito),
            | "P" => Ok(Self::Pillbug),
            | "Q" => Ok(Self::Queen),
            | "S" => Ok(Self::Spider),
            | _ => Err(Error::for_parse::<Self>(s.into())),
        }
    }
}

impl From<u8> for Bug
{
    fn from(value: u8) -> Self
    {
        let v = value.clamp(0, piece::consts::COUNT / 2 - 1);
        Bug::all().into_iter().rev().find(|kind| kind.offset() <= v).unwrap()
    }
}

impl Bug
{
    /// Returns the bugs in offset order.
    pub fn all() -> [Bug; 8]
    {
        [
            Self::Ant,
            Self::Beetle,
            Self::Grasshopper,
            Self::Ladybug,
            Self::Mosquito,
            Self::Pillbug,
            Self::Queen,
            Self::Spider,
        ]
    }

    /// Gets the extent of this bug.
    pub fn extent(&self) -> u8
    {
        match self
        {
            | Self::Ant => 3,
            | Self::Beetle => 2,
            | Self::Grasshopper => 3,
            | Self::Ladybug => 1,
            | Self::Mosquito => 1,
            | Self::Pillbug => 1,
            | Self::Queen => 1,
            | Self::Spider => 3,
        }
    }

    pub fn long(&self) -> &'static str
    {
        match self
        {
            | Self::Ant => "Ant",
            | Self::Beetle => "Beetle",
            | Self::Grasshopper => "Grasshopper",
            | Self::Ladybug => "Ladybug",
            | Self::Mosquito => "Mosquito",
            | Self::Pillbug => "Pillbug",
            | Self::Queen => "Queen",
            | Self::Spider => "Spider",
        }
    }

    /// Gets the index for this bug.
    pub fn offset(&self) -> u8
    {
        match self
        {
            | Self::Ant => 0,
            | Self::Beetle => 3,
            | Self::Grasshopper => 5,
            | Self::Ladybug => 8,
            | Self::Mosquito => 9,
            | Self::Pillbug => 10,
            | Self::Queen => 11,
            | Self::Spider => 12,
        }
    }

    /// Whether or not this piece is unique.
    pub fn unique(&self) -> bool
    {
        matches!(self, Self::Ladybug | Self::Mosquito | Self::Pillbug | Self::Queen)
    }
}
