use std::ops::{Add, Sub};

mod axial;
mod direction;
mod field;

pub use axial::Axial;
pub use direction::Direction;
pub use field::{Field, Perimeter};

/// Represents a point on a hexagonal grid.
pub type Hex = u16;

/// Values that bound the maximum size of a game of Hive.
///
/// The rules of Hive do not specify an upper bound, but
/// one is necessary to achieve an efficient implementation.
pub mod consts
{
    use super::Hex;

    const FACT: Hex = 5;
    const _FACT_FITS: () = assert!(FACT <= 8);

    pub const ROWS: Hex = 2u16.pow(FACT as u32);
    pub const SIZE: Hex = ROWS * ROWS;
    pub const MASK: Hex = SIZE.wrapping_sub(1);

    /// The starting hex of the game, used instead of the origin.
    pub const ROOT: Hex = ROWS / 2 * (ROWS + 1);

    /// The wrapping boundary for two-point conversions.
    pub const WRAP: Hex = ROWS / 2 - 1;
}

use consts::*;

#[inline]
/// Returns the two common neighbours between two adjacent hexes, provided the hexes are actually adjacent.
pub fn common_neighbours(a: Hex, b: Hex) -> Option<(Hex, Hex)>
{
    Direction::to(a, b).map(|direction| (a + direction.clockwise(), a + direction.counterclockwise()))
}

#[inline]
/// Gets the six neighbours of this hex in clockwise order.
pub fn neighbours(h: Hex) -> [Hex; 6]
{
    Direction::all().map(|d| h + d)
}

impl Add<Direction> for Hex
{
    type Output = Hex;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Direction) -> Self::Output
    {
        MASK & self.wrapping_add(rhs as Hex)
    }
}

impl Sub<Direction> for Hex
{
    type Output = Hex;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: Direction) -> Self::Output
    {
        MASK & self.wrapping_add(rhs.inverse() as Hex)
    }
}
