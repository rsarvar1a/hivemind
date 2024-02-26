use hex::consts::*;

use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A two-axis coordinate system to make it clearer which hexes are targeted in error messages.
pub struct Axial
{
    pub q: i8,
    pub r: i8,
}

impl std::fmt::Display for Axial
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "({: ^3},{: ^3})", self.q - self.r, self.r)
    }
}

impl From<Hex> for Axial
{
    fn from(hex: Hex) -> Self
    {
        let x = (hex.wrapping_sub(ROOT - ROWS / 2) / ROWS) as i8;
        let x = if x > WRAP as i8 { x - ROWS as i8 } else { x };

        let y = (hex.wrapping_sub(ROOT) % ROWS) as i8;
        let y = if y > WRAP as i8 { y - ROWS as i8 } else { y };

        Axial { q: y, r: x }
    }
}

impl From<Axial> for Hex
{
    fn from(value: Axial) -> Hex
    {
        ROOT.wrapping_add(ROWS.wrapping_mul(value.q as Hex)).wrapping_add(value.r as Hex)
    }
}
