use regex::Regex;

use crate::prelude::*;

#[derive(Clone, Debug)]
/// Represents a syntactically-valid move string.
///
/// You can't prove validity of a particular string in a vacuum, you need a board.
pub struct MoveString(pub(in crate::prelude::notation) String);

impl FromStr for MoveString
{
    type Err = Error;
    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err>
    {
        if s == "pass"
        {
            return Ok(MoveString(s.into()));
        }

        let re = Regex::new(r"^(?<src>(w|b)[A-Z][1-3]?)( (?<dest>.*))?$").unwrap();
        let Some(caps) = re.captures(s)
        else
        {
            return Err(Error::for_parse::<Self>(s.into()));
        };

        let piece = caps["src"].parse::<Piece>();
        let Ok(piece) = piece
        else
        {
            let err = piece.unwrap_err();
            return Err(err.chain_parse::<Self>(s.into()));
        };

        if let Some(next_to_str) = caps.name("dest")
        {
            let next_to_try = next_to_str.as_str().parse::<NextTo>();
            let Ok(next_to) = next_to_try
            else
            {
                let err = next_to_try.unwrap_err();
                return Err(err.chain_parse::<Self>(s.into()));
            };

            if piece == next_to.piece
            {
                let err_msg = format!("Source and destination pieces must not match ({}, {}).", piece, next_to.piece);
                let err = Error::new(Kind::LogicError, err_msg);
                return Err(err.chain_parse::<Self>(s.into()));
            }
        }

        Ok(MoveString(s.to_owned()))
    }
}

impl AsRef<str> for MoveString
{
    fn as_ref(&self) -> &str
    {
        self.0.as_str()
    }
}

impl std::fmt::Display for MoveString
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Represents a movement in the game of Hive.
pub enum Move
{
    Place(Piece, Option<NextTo>),
    Move(Piece, NextTo),
    Pass,
}

impl Default for Move
{
    fn default() -> Self
    {
        Move::Pass
    }
}

impl std::fmt::Display for Move
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let s: MoveString = (*self).into();
        s.fmt(f)
    }
}

impl From<Move> for MoveString
{
    fn from(value: Move) -> MoveString
    {
        let s: String = match value
        {
            | Move::Place(piece, to) => match to
            {
                | Some(to) => format!("{} {}", piece, to),
                | None => format!("{}", piece),
            },
            | Move::Move(piece, to) =>
            {
                format!("{} {}", piece, to)
            }
            | Move::Pass => "pass".into(),
        };
        MoveString(s)
    }
}

impl Move
{
    /// Attempts to disambiguate a MoveString into a Move using a board context.
    pub fn from(movestr: &MoveString, board: &Board) -> Result<Move>
    {
        if movestr.0.as_str() == "pass"
        {
            return Ok(Move::Pass);
        }

        let mut parts = movestr.0.split_terminator(' ').filter(|s| !s.is_empty());
        let piece = parts.next().unwrap().parse::<Piece>().unwrap();
        let nextto = parts.next().map(|s| s.parse::<NextTo>().unwrap());

        if board.placed(&piece)
        {
            if let Some(dest) = nextto
            {
                if !board.placed(&dest.piece)
                {
                    let err = Error::new(Kind::InvalidMove, format!("Reference piece {} is not in the hive.", dest.piece));
                    return Err(err.chain_parse::<Self>(movestr.0.to_owned()));
                }
                Ok(Move::Move(piece, dest))
            }
            else
            {
                let err = Error::new(Kind::InvalidMove, "Moving a piece requires a destination.".into());
                Err(err.chain_parse::<Self>(movestr.0.to_owned()))
            }
        }
        else if let Some(dest) = nextto
        {
            if !board.placed(&dest.piece)
            {
                let err = Error::new(Kind::InvalidMove, format!("Reference piece {} is not in the hive.", dest.piece));
                return Err(err.chain_parse::<Self>(movestr.0.to_owned()));
            }
            Ok(Move::Place(piece, Some(dest)))
        }
        else if board.turn() != 0
        {
            let err = Error::new(Kind::InvalidMove, "Omitting the destination is only possible on the first turn.".into());
            Err(err.chain_parse::<Self>(movestr.0.to_owned()))
        }
        else
        {
            Ok(Move::Place(piece, None))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Represents a relative location (relative to another piece).
pub struct NextTo
{
    pub piece:     Piece,
    pub direction: Option<Direction>,
}

impl FromStr for NextTo
{
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        let re = Regex::new(r"^(?<l>(?<dirl>[-/\\])(?<piecel>[A-Za-z1-3]{2,3}))$|^(?<r>(?<piecer>[A-Za-z1-3]{2,3})(?<dirr>[-/\\]))$|^(?<n>(?<piecen>[A-Za-z1-3]{2,3}))$").unwrap();
        let Some(caps) = re.captures(s)
        else
        {
            return Err(Error::for_parse::<Self>(s.into()));
        };

        if caps.name("n").is_some()
        {
            let piece = caps["piecen"].parse::<Piece>();
            let Ok(piece) = piece
            else
            {
                let err = piece.unwrap_err();
                return Err(err.chain_parse::<Self>(s.into()));
            };

            Ok(NextTo { piece, direction: None })
        }
        else
        {
            let on_left = caps.name("l").is_some();
            let piece_capture = if on_left { "piecel" } else { "piecer" };
            let dir_capture = if on_left { "dirl" } else { "dirr" };

            let piece = caps[piece_capture].parse::<Piece>();
            let Ok(piece) = piece
            else
            {
                let err = piece.unwrap_err();
                return Err(err.chain_parse::<Self>(s.into()));
            };

            let mut direction = None;
            if let Some(dir) = caps.name(dir_capture)
            {
                direction = Some(Direction::parse(dir.as_str(), on_left)?);
            }

            Ok(NextTo { piece, direction })
        }
    }
}

impl std::fmt::Display for NextTo
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let piece = self.piece;
        match self.direction
        {
            | Some(d) => match d.is_west()
            {
                | true => write!(f, "{}{}", d, piece),
                | false => write!(f, "{}{}", piece, d),
            },
            | None => write!(f, "{}", piece),
        }
    }
}
