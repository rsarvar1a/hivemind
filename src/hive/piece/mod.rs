use crate::prelude::*;

mod bug;
mod player;
mod pouch;

pub use bug::Bug;
pub use player::Player;
pub use pouch::Pouch;

pub mod consts
{
    pub const PER_PLAYER: u8 = 14;
    pub const COUNT: u8 = 2 * PER_PLAYER;
    pub const HEIGHT_RANGE: u8 = 8;
}
use consts::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// A piece in the game of Hive.
///
/// Pieces have a player, a bug type, and a numeric discriminator.
///
/// For example, the third Ant in white's hand is wA3.
pub struct Piece
{
    pub player: Player,
    pub kind:   Bug,
    pub num:    u8,
}

impl Piece
{
    /// Gets the index of this piece, in player-kind-num order.
    pub fn index(&self) -> u16
    {
        (PER_PLAYER * (self.player as u8) + self.kind.offset() + (self.num - 1)) as u16
    }
}

impl FromStr for Piece
{
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        if s.len() < 2 || s.len() > 3
        {
            let err = Error::new(Kind::ParseError, format!("Invalid length (expected 2 or 3, found {}).", s.len()));
            return Err(err.chain_parse::<Self>(s.into()));
        }

        let player = s[0..=0].parse::<Player>();
        let kind = s[1..=1].parse::<Bug>();

        let Ok(player) = player
        else
        {
            let err = player.unwrap_err();
            return Err(err.chain_parse::<Self>(s.into()));
        };

        let Ok(kind) = kind
        else
        {
            let err = kind.unwrap_err();
            return Err(err.chain_parse::<Self>(s.into()));
        };

        let num = if kind.unique()
        {
            if s.len() > 2
            {
                let err = Error::new(Kind::ParseError, "Unique bugs should have no number.".into());
                return Err(err.chain_parse::<Self>(s.into()));
            }
            1
        }
        else
        {
            if s.len() < 3
            {
                let err = Error::new(Kind::ParseError, "Non-unique bugs must have a number.".into());
                return Err(err.chain_parse::<Self>(s.into()));
            }

            let num_parse = s[2..=2].parse::<u8>();

            let Ok(found_num) = num_parse
            else
            {
                let err = Error::for_parse::<u8>(s[2..=2].into());
                return Err(err.chain_parse::<Self>(s.into()));
            };

            if !(1..=kind.extent()).contains(&found_num)
            {
                let err_msg = format!(
                    "Invalid number for {} (expected {} to {}, found {}).",
                    kind.long(),
                    1,
                    kind.extent(),
                    found_num
                );

                let err = Error::new(Kind::MismatchError, err_msg);
                return Err(err.chain_parse::<Self>(s.into()));
            }
            found_num
        };

        Ok(Piece { player, kind, num })
    }
}

impl From<u8> for Piece
{
    fn from(value: u8) -> Self
    {
        // Find the player, and regularize the index to the kind-num range.

        let mut v = value.clamp(0, 27);
        let player = if value <= 13
        {
            Player::White
        }
        else
        {
            v -= 14;
            Player::Black
        };
        let v = v;

        // Find the bug from the given value.

        let kind: Bug = v.into();
        let num = v - kind.offset() + 1;

        Piece { player, kind, num }
    }
}

impl std::fmt::Display for Piece
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self.kind.unique()
        {
            | true => write!(f, "{}{}", self.player.short(), self.kind),
            | false => write!(f, "{}{}{}", self.player.short(), self.kind, self.num),
        }
    }
}
