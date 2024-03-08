use crate::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// A wrapped type for a move.
///
/// Bits:
///     00 - 04: piece
///     05 - 05: piece.is_some()
///     06 - 0A: nextto.piece
///     0B - 1A: nextto.direction
///     1B - 1B: nextto.direction.is_some()
///     1C - 1C: nextto.is_some()
///     1D - 1E: move::enum_type [Pass, Place, Move]
///     1F - 1F: move.is_some()
pub struct MoveToken(u32);

impl From<Move> for MoveToken
{
    fn from(value: Move) -> Self
    {
        match value
        {
            | Move::Move(piece, nextto) =>
            {
                let set = 1u32 << Self::OFFSET_MOVE_OPT;
                let p__ = (piece.index() as u32) << Self::OFFSET_PIECE;
                let po_ = 1u32 << Self::OFFSET_PIECE_OPT;
                let np_ = (nextto.piece.index() as u32) << Self::OFFSET_NEXTTO_PIECE;
                let nd_ = (nextto.direction.unwrap_or(Direction::East) as u32) << Self::OFFSET_NEXTTO_DIRECTION;
                let ndo = (nextto.direction.is_some() as u32) << Self::OFFSET_NEXTTO_DIRECTION_OPT;
                let no_ = 1u32 << Self::OFFSET_NEXTTO_OPT;
                let et_ = Self::TYPE_MOVE << Self::OFFSET_ENUM_TYPE;

                MoveToken(set | p__ | po_ | np_ | nd_ | ndo | no_ | et_)
            }
            | Move::Place(piece, nextto) =>
            {
                let set = 1u32 << Self::OFFSET_MOVE_OPT;
                let p__ = (piece.index() as u32) << Self::OFFSET_PIECE;
                let po_ = 1u32 << Self::OFFSET_PIECE_OPT;
                let np_ = (nextto.map(|n| n.piece.index()).unwrap_or(0) as u32) << Self::OFFSET_NEXTTO_PIECE;
                let nd_ = (nextto.map(|n| n.direction.unwrap_or(Direction::East)).unwrap_or(Direction::East) as u32) << Self::OFFSET_NEXTTO_DIRECTION;
                let ndo = (nextto.map(|n| n.direction).unwrap_or(None).is_some() as u32) << Self::OFFSET_NEXTTO_DIRECTION_OPT;
                let no_ = (nextto.is_some() as u32) << Self::OFFSET_NEXTTO_OPT;
                let et_ = Self::TYPE_PLACE << Self::OFFSET_ENUM_TYPE;

                MoveToken(set | p__ | po_ | np_ | nd_ | ndo | no_ | et_)
            }
            | Move::Pass => 
            {
                let set = 1u32 << Self::OFFSET_MOVE_OPT;
                let et_ = Self::TYPE_PASS << Self::OFFSET_ENUM_TYPE;

                MoveToken(set | et_)
            }
        }
    }
}

impl From<MoveToken> for Option<Move>
{
    fn from(value: MoveToken) -> Self
    {
        match value.is_some()
        {
            | true => Some(match value.enum_type()
            {
                | MoveToken::TYPE_PASS => Move::Pass,
                | MoveToken::TYPE_MOVE =>
                {
                    let piece = value.piece().unwrap();
                    let nextto = value.nextto().unwrap();
                    Move::Move(piece, nextto)
                }
                | MoveToken::TYPE_PLACE =>
                {
                    let piece = value.piece().unwrap();
                    let nextto = value.nextto();
                    Move::Place(piece, nextto)
                }
                | _ => unreachable!(),
            }),
            | false => None,
        }
    }
}

impl MoveToken
{
    const OFFSET_PIECE: u32 = 0x0;
    const OFFSET_PIECE_OPT: u32 = 0x5;
    const OFFSET_NEXTTO_PIECE: u32 = 0x6;
    const OFFSET_NEXTTO_DIRECTION: u32 = 0xB;
    const OFFSET_NEXTTO_DIRECTION_OPT: u32 = 0x1B;
    const OFFSET_NEXTTO_OPT: u32 = 0x1C;
    const OFFSET_ENUM_TYPE: u32 = 0x1D;
    const OFFSET_MOVE_OPT: u32 = 0x1F;

    const EXTENT_DIRECTION: u32 = u16::MAX as u32;
    const EXTENT_ENUM_TYPE: u32 = 0b11;
    const EXTENT_OPTION: u32 = 0b1;
    const EXTENT_PIECE: u32 = 0b11111;

    const TYPE_PASS: u32 = 0x0;
    const TYPE_MOVE: u32 = 0x1;
    const TYPE_PLACE: u32 = 0x2;

    /// Extracts the direction in a NextTo provided one exists.
    fn direction(&self) -> Option<Direction>
    {
        if self.is_set(Self::OFFSET_NEXTTO_DIRECTION_OPT)
        {
            let val = ((self.0 >> Self::OFFSET_NEXTTO_DIRECTION) & Self::EXTENT_DIRECTION) as Hex;
            let direction = unsafe { std::mem::transmute(val) };
            Some(direction)
        }
        else
        {
            None
        }
    }

    /// Extracts the type.
    fn enum_type(&self) -> u32
    {
        self.0 >> Self::OFFSET_ENUM_TYPE & Self::EXTENT_ENUM_TYPE
    }

    /// Ensures the given option is set.
    fn is_set(&self, offset: u32) -> bool
    {
        ((self.0 >> offset) & Self::EXTENT_OPTION) == 1
    }

    /// Whether or not this move is really a move.
    pub fn is_some(&self) -> bool
    {
        self.is_set(Self::OFFSET_MOVE_OPT)
    }

    /// Extracts the reference hex.
    fn nextto(&self) -> Option<NextTo>
    {
        if self.is_set(Self::OFFSET_NEXTTO_OPT)
        {
            let piece: Piece = (((self.0 >> Self::OFFSET_NEXTTO_PIECE) & Self::EXTENT_PIECE) as u8).into();

            let direction = if self.is_set(Self::OFFSET_NEXTTO_DIRECTION_OPT)
            {
                let dir: Direction = self.direction().unwrap();
                Some(dir)
            }
            else
            {
                None
            };

            Some(NextTo { piece, direction })
        }
        else
        {
            None
        }
    }

    /// Extracts the piece.
    fn piece(&self) -> Option<Piece>
    {
        if self.is_set(Self::OFFSET_PIECE_OPT)
        {
            let index: u8 = ((self.0 >> Self::OFFSET_PIECE) & Self::EXTENT_PIECE) as u8;
            Some(index.into())
        }
        else
        {
            None
        }
    }
}
