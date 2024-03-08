use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A utility representation of a movement to store changes to hexes.
pub struct Patch
{
    pub piece: Piece,
    pub from:  Option<Hex>,
    pub to:    Hex,
}

impl std::fmt::Display for Patch
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let from = self.from.map(|hex| format!("{}", Axial::from(hex))).unwrap_or("pouch".into());
        write!(f, "{: <3} to {}, from {}", format!("{}", self.piece), Axial::from(self.to), from)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// A movement-patch pair for easy backward restoration.
pub struct Entry
{
    pub mv:           Move,
    pub patch:        Option<Patch>,
    pub prev_stunned: Option<Hex>,
}

impl std::fmt::Debug for Entry
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let patch = self.patch.map(|p| format!("{}", p)).unwrap_or("none".into());
        write!(f, "(move: {: <9}, patch: {})", format!("{}", self.mv), patch)
    }
}

#[derive(Clone, Debug, Default)]
/// A linear move history.
///
/// The history can undo moves from the present back to the start.
///
/// It can also redo moves until a new move is made at any point in the history.
pub struct History
{
    past:   Vec<Entry>,
    future: Vec<Entry>,
}

impl History
{
    /// Gets the in-order past of this history.
    pub fn get_past(&self) -> Vec<Entry>
    {
        let mut v = self.past.clone();
        v.reverse();
        v
    }

    /// Determines whether or not the history is empty.
    pub fn is_empty(&self) -> bool
    {
        self.len() == 0
    }

    /// A read-only iter to past moves.
    pub fn iter(&self) -> std::slice::Iter<'_, Entry>
    {
        self.past.iter()
    }

    /// Finds the most recent destination for a piece, if one exists.
    /// This is useful for undoing moves, because you need to know where
    /// the piece came from in the Hive (or possibly if it was in the hand).
    pub fn last_hex(&self, piece: Piece) -> Option<Hex>
    {
        let iter = self.past.iter().rev().find(|p| p.patch.is_some() && p.patch.unwrap().piece == piece);

        iter.map(|p| p.patch.unwrap().to)
    }

    /// Gets the length of the history, which is useful for controlling undos.
    pub fn len(&self) -> usize
    {
        self.past.len()
    }

    #[allow(clippy::should_implement_trait)]
    /// Gets the next move to be played in this line, if one exists.
    pub fn next(&self) -> Option<Entry>
    {
        self.future.last().copied()
    }

    /// Plays a move, which clears the future if this move doesn't match.
    pub fn play(&mut self, entry: Entry)
    {
        let next = self.future.last().copied();
        if let Some(mv) = next
        {
            if mv == entry
            {
                // We are consistent with the history (the move is a redo)
                // so we can just step forwards like normal.
                self.redo();
            }
            else
            {
                // Otherwise, we broke the linear history, so it needs to be cleared.
                self.future.clear();
                self.past.push(entry);
            }
        }
        else
        {
            self.past.push(entry);
        }
    }

    /// Gets the last move played in this line, if one exists.
    pub fn prev(&self) -> Option<Entry>
    {
        self.past.last().copied()
    }

    /// Steps forward in the history if possible.
    pub fn redo(&mut self)
    {
        if let Some(mv) = self.next()
        {
            self.future.pop();
            self.past.push(mv);
        }
    }

    /// The turn number is the number of moves already played.
    ///
    /// A new game therefore begins at 0.
    pub fn turn(&self) -> u8
    {
        self.past.len() as u8
    }

    /// Steps backward in the history if possible.
    pub fn undo(&mut self)
    {
        if let Some(mv) = self.prev()
        {
            self.past.pop();
            self.future.push(mv);
        }
    }
}
