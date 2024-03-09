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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// A drop-in replacement specifically for HashSet<Hex> where the caller only needs to check set membership.
///
/// If you need to iterate, use a real collection.
pub struct Collection([u64; Self::SIZE as usize]);

impl Collection
{
    const SIZE: u64 = consts::SIZE as u64 / 64;
    const MASK: u64 = Self::SIZE - 1;
    const SHFT: u64 = (consts::SIZE as u64).trailing_zeros() as u64 - 6;

    /// Determines whether or not this collection contains the given set.
    pub fn contains(&self, hex: Hex) -> bool
    {
        let index = self.index_into_list(hex);
        let read = self.index_into_word(hex);
        (self.0[index] >> read) & 1 != 0
    }

    /// Inserts the given hex into the set.
    pub fn insert(&mut self, hex: Hex)
    {
        let index = self.index_into_list(hex);
        let write = 1 << self.index_into_word(hex);
        self.0[index] |= write;
    }

    /// Returns a new, empty hex set.
    pub fn new() -> Self
    {
        Collection([0; Self::SIZE as usize])
    }

    /// Removes the given hex from the set.
    pub fn remove(&mut self, hex: Hex)
    {
        let index = self.index_into_list(hex);
        let write = !(1 << self.index_into_word(hex));
        self.0[index] &= write;
    }

    fn index_into_list(&self, hex: Hex) -> usize
    {
        hex as usize & Self::MASK as usize
    }

    fn index_into_word(&self, hex: Hex) -> u64
    {
        hex as u64 >> Self::SHFT
    }
}
