pub mod depth;
pub mod scores;

pub use depth::*;
pub use scores::consts::*;

/// The maximum permitted depth to search to, and the maximum size of a continuation.
pub const MAXIMUM_PLY: usize = Depth::MAX.floor() as usize;

/// An integer-friendly infinity.
pub const INF: i32 = i16::MAX as i32 + 1;

/// An integer-friendly NaN.
pub const NAN: i32 = INF + 1;
