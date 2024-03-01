use std::{
    hash::{DefaultHasher, Hasher},
    u128,
};

use lazy_static::lazy_static;

use crate::prelude::*;

const HEIGHTS: usize = piece::consts::HEIGHT_RANGE as usize;
const HEXES: usize = hex::consts::SIZE as usize;
const PIECES: usize = piece::consts::COUNT as usize;

/// We need a bitstring for each piece on each hex at each height.
const NUM_BITSTRINGS: usize = HEIGHTS * HEXES * PIECES;
const BITSTRING_MASK: u128 = u16::MAX as u128;

const OFFSET_LAST: usize = 0x40;
const OFFSET_LAST_VALID: usize = 0x61;

const OFFSET_STUN: usize = 0x50;
const OFFSET_STUN_VALID: usize = 0x62;

const OFFSET_PLAYER: usize = 0x60;
const EXTENT_PLAYER: u128 = 0x1;

const EXTENT_HEX: usize = u16::MAX as usize;
const EXTENT_OPT: usize = 1;

lazy_static! {
    /// A *BIG* table of bitstrings used by the Zobrist calculations.
    ///
    /// Instantiating this table takes around the order of 2MB, but only once.
    ///
    /// Don't worry... it will be dwarfed by the transposition table! :)
    static ref BITSTRINGS: Box<[u64; NUM_BITSTRINGS]> =
    {
        let mut table = Box::new([0u64; NUM_BITSTRINGS]);
        let mut hasher = DefaultHasher::new();

        for i in 0 .. table.len()
        {
            hasher.write_u64(i as u64);
            table[i] = hasher.finish();
        }
        table
    };
}

#[derive(Clone, Copy, Debug, Default)]
/// An implementor for zobrist hashes.
pub struct ZobristTable
{
    /// The hash being operated on.
    current: ZobristHash,
}

impl ZobristTable
{
    /// Gets the hash associated with the current state.
    pub fn get(&self) -> ZobristHash
    {
        self.current
    }

    /// Hashes a piece into or out of a particular hex. The operation is symmetric.
    pub fn hash(&mut self, piece: &Piece, at: Hex, height: u8) -> &mut Self
    {
        let location = HEIGHTS * at as usize + height as usize;
        let index: usize = PIECES * location + piece.index() as usize;

        let bitstring: u64 = BITSTRINGS[index];
        self.current ^= bitstring as u128 & BITSTRING_MASK;
        self
    }

    /// Sets the last destination to the given hex to track pillbug immunity.
    pub fn last(&mut self, to: Option<Hex>) -> &mut Self
    {
        self.current &= (!(EXTENT_HEX as u128)).rotate_left(OFFSET_LAST as u32);
        self.current |= (to.unwrap_or(0) as u128) << OFFSET_LAST;

        self.current &= (!(EXTENT_OPT as u128)).rotate_left(OFFSET_LAST_VALID as u32);
        self.current |= (to.is_some() as u128) << OFFSET_LAST_VALID;

        self
    }

    #[allow(clippy::should_implement_trait)]
    /// Advances to the next player to move.
    pub fn next(&mut self) -> &mut Self
    {
        let prev = to_move(self.current);
        self.player(prev.flip())
    }

    /// Declares the player to move.
    pub fn player(&mut self, player: Player) -> &mut Self
    {
        self.current |= (player as u128) << OFFSET_PLAYER;
        self
    }

    /// Reverses to the previous player to move.
    pub fn prev(&mut self) -> &mut Self
    {
        self.next()
    }

    /// Sets the stun destination to track the hex last touched by the Pillbug.
    pub fn stun(&mut self, to: Option<Hex>) -> &mut Self
    {
        self.current &= (!(EXTENT_HEX as u128)).rotate_left(OFFSET_STUN as u32);
        self.current |= (to.unwrap_or(0) as u128) << OFFSET_STUN;

        self.current &= (!(EXTENT_OPT as u128)).rotate_left(OFFSET_STUN_VALID as u32);
        self.current |= (to.is_some() as u128) << OFFSET_STUN_VALID;

        self
    }
}

/// A densely-packed zobrist hash.
///
/// Bits:
///
/// 00 - 3F: 64-bit tokenfield hash (representing 8192 tokens)
///
/// 40 - 4F: last destination hex (for pillbug immunity)
///
/// 50 - 50: player to move
pub type ZobristHash = u128;

/// Determines which player needs to move.
pub fn to_move(hash: ZobristHash) -> Player
{
    let bits = ((hash >> OFFSET_PLAYER) & EXTENT_PLAYER) as u8;
    unsafe { std::mem::transmute(bits) }
}
