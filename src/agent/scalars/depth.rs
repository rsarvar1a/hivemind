use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A ply-representation of search depth.
pub struct Depth(i32);

impl From<u8> for Depth
{
    fn from(value: u8) -> Self
    {
        Depth::new(value as i32)
    }
}

impl From<Depth> for i32
{
    fn from(value: Depth) -> Self
    {
        value.0
    }
}

impl Depth
{
    /// The resolution of a depth.
    const PER_PLY: Depth = Depth(100);

    /// Zero plies.
    pub const NIL: Depth = Depth::new(0);

    /// One ply.
    pub const PLY: Depth = Depth::new(1);

    /// The max search depth is 128 plys, or 64 turns.
    pub const MAX: Depth = Depth::new(1 + i8::MAX as i32);

    /// Clamps this depth to a valid ply. If it was negative, clamps to 0.
    pub const fn clamp(&self) -> i32
    {
        if self.0 <= 0
        {
            0
        }
        else
        {
            self.floor()
        }
    }

    /// Whether this depth represents an exact ply.
    pub const fn exact(&self) -> bool
    {
        self.0 % Self::PER_PLY.0 == 0
    }

    /// Rounds down to the exact ply.
    pub const fn floor(&self) -> i32
    {
        self.0 / Self::PER_PLY.0
    }

    /// Constructs a new depth.
    pub const fn new(value: i32) -> Depth
    {
        Depth(value * Self::PER_PLY.0)
    }

    /// Constructs a raw depth, which does not take into account internal multipliers, and just loads the value.
    pub const fn raw(value: i32) -> Depth
    {
        Depth(value)
    }

    /// Rounds to the nearest exact ply as a depth.
    pub const fn rounded(&self) -> Depth
    {
        let x = self.0 + Self::PER_PLY.0 / 2;
        Depth(x - x % Self::PER_PLY.0)
    }

    /// Returns the square of the depth.
    pub const fn squared(&self) -> i32
    {
        self.0 * self.0 / Self::PER_PLY.0 / Self::PER_PLY.0
    }

    /// Whether or not this is a valid depth; that is, it is smaller than the maximum ply count.
    pub const fn valid(&self) -> bool
    {
        0 <= self.0 && self.0 <= Self::MAX.0
    }
}

impl Add<Self> for Depth
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output
    {
        Depth(self.0 + rhs.0)
    }
}

impl Add<i32> for Depth
{
    type Output = Self;
    fn add(self, rhs: i32) -> Self::Output
    {
        Depth(self.0 + rhs * Self::PER_PLY.0)
    }
}

impl AddAssign<Self> for Depth
{
    fn add_assign(&mut self, rhs: Self)
    {
        *self = *self + rhs;
    }
}

impl AddAssign<i32> for Depth
{
    fn add_assign(&mut self, rhs: i32)
    {
        *self = *self + rhs;
    }
}

impl Sub<Self> for Depth
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output
    {
        Depth(self.0 - rhs.0)
    }
}

impl Sub<i32> for Depth
{
    type Output = Self;
    fn sub(self, rhs: i32) -> Self::Output
    {
        Depth(self.0 - rhs * Self::PER_PLY.0)
    }
}

impl SubAssign<Self> for Depth
{
    fn sub_assign(&mut self, rhs: Self)
    {
        *self = *self - rhs;
    }
}

impl SubAssign<i32> for Depth
{
    fn sub_assign(&mut self, rhs: i32)
    {
        *self = *self - rhs;
    }
}

impl Mul<Self> for Depth
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output
    {
        self * rhs.0 / Self::PER_PLY.0
    }
}

impl Mul<i32> for Depth
{
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output
    {
        Depth(self.0 * rhs)
    }
}

impl Div<i32> for Depth
{
    type Output = Self;
    fn div(self, rhs: i32) -> Self::Output
    {
        Depth(self.0 / rhs)
    }
}

impl Neg for Depth
{
    type Output = Self;
    fn neg(self) -> Self::Output
    {
        Depth(-self.0)
    }
}

impl std::iter::Step for Depth
{
    fn backward(start: Self, count: usize) -> Self
    {
        start - (count as i32)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self>
    {
        let d = Self::backward(start, count);
        if d < Self::NIL
        {
            None
        }
        else
        {
            Some(d)
        }
    }

    fn forward(start: Self, count: usize) -> Self
    {
        start + (count as i32)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self>
    {
        let d = Self::forward(start, count);
        if d > Self::MAX
        {
            None
        }
        else
        {
            Some(d)
        }
    }

    fn steps_between(start: &Self, end: &Self) -> Option<usize>
    {
        if start > end
        {
            return None;
        }

        let diff = *end - *start;

        match diff.exact()
        {
            | true => Some(diff.floor() as usize),
            | false => None,
        }
    }
}
