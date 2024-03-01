use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
/// A collection of pieces that have not yet entered play.
pub struct Pouch
{
    pieces: [[u8; 8]; 2],
    totals: [u8; 8],
}

impl Default for Pouch
{
    fn default() -> Self
    {
        let options = Options::default();
        Pouch::new(options)
    }
}

impl Pouch
{
    /// Creates a new pouch with optional bugs determined by the game options.
    pub fn new(options: Options) -> Pouch
    {
        let extents = Pouch::_extents(options);
        Pouch {
            pieces: [extents, extents],
            totals: extents,
        }
    }

    /// Returns the starting number of each bug for the options set on this pouch.
    pub fn extents(&self) -> &[u8; 8]
    {
        &self.totals
    }

    /// Determines how many bugs of each type are left in a player's hand.
    pub fn hand(&self, player: Player) -> &[u8; 8]
    {
        &self.pieces[player as usize]
    }

    /// Peeks the next bug.
    pub fn next(&self, player: Player, kind: Bug) -> Option<Piece>
    {
        self.peek(player, kind).map(|num| Piece { player, kind, num })
    }

    /// Returns the lowest discriminator left for the given piece type.
    pub fn peek(&self, player: Player, kind: Bug) -> Option<u8>
    {
        let remaining = self.pieces[player as usize][kind as usize];
        if remaining > 0
        {
            let discrim = 1 + self.totals[kind as usize] - remaining;
            Some(discrim)
        }
        else
        {
            None
        }
    }

    /// Puts a piece back into the bag. the discriminator is unchecked, so you should
    /// get the correct one using peek before using this method.
    pub fn put(&mut self, piece: Piece)
    {
        self.pieces[piece.player as usize][piece.kind as usize] += 1;
    }

    /// Tries to take a piece from the bag if one still exists. Returns the lowest
    /// discriminator that remains for the requested bug.
    pub fn take(&mut self, player: Player, kind: Bug) -> Option<Piece>
    {
        let next = self.peek(player, kind);
        if let Some(num) = next
        {
            self.pieces[player as usize][kind as usize] -= 1;
            Some(Piece { player, kind, num })
        }
        else
        {
            None
        }
    }

    /// Returns the starting number of each bug.
    fn _extents(options: Options) -> [u8; 8]
    {
        let exp = options.expansions;

        let mut base: [u8; 8] = [3, 2, 3, 0, 0, 0, 1, 2];
        let mask: [u8; 8] = [0, 0, 0, exp.ladybug as u8, exp.mosquito as u8, exp.pillbug as u8, 0, 0];
        for i in 0..8
        {
            base[i] += mask[i];
        }

        base
    }
}
