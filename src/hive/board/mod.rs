use std::collections::HashSet;

use crate::prelude::*;

mod checks;
mod ensures;
mod generate;
mod history;
mod printers;
mod state;
mod token;
mod zobrist;

use history::Patch;
pub use history::{Entry, History};
pub use state::GameState;
pub use token::{Stack, Token};
pub use zobrist::ZobristHash;
use zobrist::ZobristTable;

pub mod consts
{
    use super::hex;
    pub const SIZE: usize = hex::consts::SIZE as usize;
    pub const PIECES: usize = crate::prelude::piece::consts::COUNT as usize;
}

use consts::*;

#[derive(Clone)]
/// A fast board for fast state computation.
pub struct Board
{
    /// We also keep a fast set of filled positions for fast indexing and graph computations.
    field: Field,

    /// The linear history on this board, which is a forward stack paired with a backward stack.
    history: History,

    /// The hex immune to the Pillbug.
    ///
    /// A hex is immune when it was moved/placed by the opponent on the previous turn.
    immune: Option<Hex>,

    /// The options that apply to this game, such as its expansions and tournament settings.
    options: Options,

    /// THe locations of each piece for shorthand purposes.
    pieces: [Option<Hex>; PIECES],

    /// The set of pinned hexes.
    pinned: Collection,

    /// Stores the pieces that have not yet been put on the board.
    pouch: Pouch,

    /// The board is just an array of token stacks that might or might not be filled.
    ///
    /// The maximum height of any stack is 7, which corresponds to 4 beetles, 2 mosquitos as beetles, and an underlying bug.
    ///
    /// We waste a bit of memory to use the 0th index to store tokens that are just height markers, then index into the
    /// stacks with the 1-indexed token heights instead of the 0-indexed heights.
    stacks: [Stack; SIZE],

    /// The hex stunned by the directly preceding Pillbug move.
    stunned: Option<Hex>,

    /// A utility to calculate zobrist hashes for this board.
    ///
    /// It does not own the transposition table; that responsibility is left to the agent.
    zobrist: ZobristTable,
}

impl PartialEq for Board
{
    fn eq(&self, other: &Self) -> bool
    {
        // Same GameType
        self.options.expansions == other.options.expansions
            // Same Pillbug states
            && self.immune == other.immune
            && self.stunned == other.stunned
            // Same board configuration; most expensive
            && self.stacks == other.stacks
    }
}

impl Eq for Board {}

impl Default for Board
{
    fn default() -> Self
    {
        let options = Options::default();
        Board::new(options)
    }
}

impl std::fmt::Debug for Board
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if f.alternate()
        {
            self.pretty(f)
        }
        else
        {
            self.debug(f)
        }
    }
}

impl std::fmt::Display for Board
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Board")?;
        let hexes: HashSet<Hex> = self.field.clone().into();
        for hex in hexes
        {
            let axial = Axial::from(hex);
            let stack = self.stacks[hex as usize];
            write!(f, "\n\t{}: {}", axial, stack)?;
        }
        Ok(())
    }
}

impl std::hash::Hash for Board
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H)
    {
        // Extremely fast.
        self.zobrist.get().hash(state)
    }
}

impl Board
{
    /// Ensures a move is valid in the current position, or returns an error explaining why it isn't.
    pub fn check(&self, mv: &Move) -> Result<()>
    {
        match mv
        {
            | Move::Place(piece, nextto) =>
            {
                let hex = self.resolve(nextto);
                self.can_place(piece, hex)
            }
            | Move::Move(piece, nextto) =>
            {
                let hex = self.resolve(&Some(*nextto));
                self.can_move(piece, hex)
            }
            | Move::Pass => Ok(()),
        }
    }

    /// Returns the field of this hive.
    pub fn field(&self) -> &Field
    {
        &self.field
    }

    /// Generates non-pillbug or non-mosquito-as-pillbug moves.
    pub fn generate_non_throws(&self, standard_position: bool) -> Vec<Move>
    {
        let mut moves: Vec<Move> = Vec::new();
        self.generate_placements_into(standard_position, &mut moves);
        self.generate_moves_into(&mut moves);
        moves
    }

    /// Gets the history of this game.
    pub fn history(&self) -> &History
    {
        &self.history
    }

    /// Returns the immune hex, if one exists.
    pub fn immune(&self) -> Option<Hex>
    {
        self.immune
    }

    /// Check if a ground-level hex is blocked at ground level.
    pub fn is_blocked_crawler(&self, hex: Hex) -> bool
    {
        if self.stacks[hex as usize].height() != 1
        {
            return false;
        }

        if self.pinned.contains(hex)
        {
            return true;
        }

        let neighbours = hex::neighbours(hex);
        let mut open = HashSet::new();

        for neighbour in neighbours
        {
            if !self.occupied(neighbour)
            {
                open.insert(neighbour);
            }
        }

        if open.len() <= 2
        {
            return false;
        }

        for pair in itertools::iproduct!(open.iter(), open.iter())
        {
            // If two neighbouring spots are open, the bug is definitely not blocked.
            if hex::common_neighbours(*pair.0, *pair.1).is_some()
            {
                return true;
            }
        }

        false
    }

    /// Determines if the given hex is pinned.
    pub fn is_pinned(&self, piece: &Piece) -> bool
    {
        let Some(hex) = self.pieces[piece.index() as usize]
        else
        {
            return false;
        };

        let stack = self.stacks[hex as usize];
        let token = self.top(hex).unwrap();

        if token != *piece
        {
            true
        }
        else if stack.height() > 1
        {
            false
        }
        else
        // stack.height() == 1
        {
            self.pinned.contains(hex)
        }
    }

    /// Returns the hex that this piece is on, if any.
    pub fn location(&self, piece: &Piece) -> Option<Hex>
    {
        self.pieces[piece.index() as usize]
    }

    /// Returns the pieces neighbouring a given hex.
    pub fn neighbours(&self, hex: Hex) -> HashSet<Piece>
    {
        self.field
            .neighbours(hex)
            .into_iter()
            .filter_map(|h| self.top(h))
            .collect::<HashSet<Piece>>()
    }

    // Creates a new unstarted board with the given options.
    pub fn new(options: Options) -> Board
    {
        Board {
            field: Field::default(),
            history: History::default(),
            immune: None,
            options,
            pieces: [None; PIECES],
            pinned: Collection::new(),
            pouch: Pouch::new(options),
            stacks: [Stack::default(); SIZE],
            stunned: None,
            zobrist: ZobristTable::default(),
        }
    }

    /// Determines whether or not a piece is at this hex.
    pub fn occupied(&self, hex: Hex) -> bool
    {
        !self.stacks[hex as usize].empty()
    }

    /// Determines whether or not the given piece is in the Hive and at the top of whichever stack it is in.
    pub fn on_top(&self, piece: &Piece) -> bool
    {
        let token: Token = (*piece).into();
        let location = self.pieces[piece.index() as usize];

        match location
        {
            | Some(hex) =>
            {
                let stack: Stack = self.stacks[hex as usize];
                stack.top() == token
            }
            | None => false,
        }
    }

    /// Gets the options configured for this game.
    pub fn options(&self) -> Options
    {
        self.options
    }

    /// Returns all of the hexes on the "outside" of the Hive.
    pub fn perimeter(&self, as_if_removing: Option<Hex>) -> Perimeter
    {
        self.field.perimeter(as_if_removing)
    }

    /// Returns the pieces neighbouring a particular hex.
    pub fn pieces_neighbouring(&self, hex: Hex) -> HashSet<Piece>
    {
        hex::neighbours(hex).into_iter().filter_map(|hex| self.top(hex)).collect()
    }

    /// Gets all pieces placed in the hive that are also pinned.
    pub fn pinned_pieces(&self, player: Player) -> HashSet<Piece>
    {
        self.pieces
            .iter()
            .enumerate()
            .flat_map(|(i, hex)| hex.map(|_| Piece::from(i as u8)))
            .filter(|piece| piece.player == player && self.is_pinned(piece))
            .collect()
    }

    /// Gets all pieces pinned.
    pub fn pinned_pieces_all(&self) -> HashSet<Piece>
    {
        self.pieces
            .iter()
            .enumerate()
            .flat_map(|(i, hex)| hex.map(|_| Piece::from(i as u8)))
            .filter(|piece| self.is_pinned(piece))
            .collect()
    }

    // Gets all pinned hexes.
    pub fn pinned_hexes(&self) -> Collection
    {
        self.pinned.clone()
    }

    /// Determines whether or not the given bug is already in the Hive.
    pub fn placed(&self, piece: &Piece) -> bool
    {
        self.pieces[piece.index() as usize].is_some()
    }

    /// Plays the given move on the board, if possible.
    ///
    /// Returns the hash of the new position.
    pub fn play(&mut self, mv: &Move) -> Result<ZobristHash>
    {
        self.check(mv)?;
        Ok(self.play_unchecked(mv))
    }

    /// Gets the pouch for this game.
    pub fn pouch(&self) -> &Pouch
    {
        &self.pouch
    }

    /// Finds the queen for the given player.
    pub fn queen(&self, player: Player) -> Option<Hex>
    {
        self.pieces[Piece {
            player,
            kind: Bug::Queen,
            num: 1,
        }
        .index() as usize]
    }

    /// If there is a future move in this line, replays it.
    pub fn redo(&mut self) -> Result<ZobristHash>
    {
        let Some(entry) = self.history.next()
        else
        {
            let err = Error::new(Kind::InvalidMove, "No move to redo".into());
            let base = Error::new(
                Kind::InternalError,
                format!("Failed to redo move on GameString {}.", GameString::from(&*self)),
            );
            return Err(Error::holy_shit(err.chain(base)));
        };
        Ok(self.play_unchecked(&entry.mv))
    }

    /// Deteremines whether or not this bug is stacked.
    ///
    /// A stacked bug is a bug in a stack of any height greater than 1.
    pub fn stacked(&self, piece: &Piece) -> bool
    {
        match self.pieces[piece.index() as usize]
        {
            | Some(hex) => self.stacks[hex as usize].height() > 1,
            | None => false,
        }
    }

    /// Gets the state of the board.
    pub fn state(&self) -> GameState
    {
        if self.turn() == 0
        {
            return GameState::NotStarted;
        }

        let white_hex = self.queen(Player::White);
        let white_surrounded = white_hex.is_some() && hex::neighbours(white_hex.unwrap()).iter().all(|h| self.field.contains(*h));

        let black_hex = self.queen(Player::Black);
        let black_surrounded = black_hex.is_some() && hex::neighbours(black_hex.unwrap()).iter().all(|h| self.field.contains(*h));

        match (white_surrounded, black_surrounded)
        {
            | (false, false) => GameState::InProgress,
            | (false, true) => GameState::WhiteWins,
            | (true, false) => GameState::BlackWins,
            | (true, true) => GameState::Draw,
        }
    }

    /// Returns the stunned hex, if one exists.
    pub fn stunned(&self) -> Option<Hex>
    {
        self.stunned
    }

    // Returns the player that should play the next move.
    pub fn to_move(&self) -> Player
    {
        let turn: Turn = self.turn().into();
        turn.player
    }

    /// Gets the piece visible at the top of the given stack.
    pub fn top(&self, hex: Hex) -> Option<Piece>
    {
        self.stacks[hex as usize].top().into()
    }

    /// Gets the turn number, which is the number of moves already played.
    ///
    /// The turn number on a new board is therefore 0, which maps to `White[1]` as a turn string.
    pub fn turn(&self) -> u8
    {
        self.history.turn()
    }

    /// Undoes a number of moves, if possible.
    pub fn undo(&mut self, n: u8) -> Result<ZobristHash>
    {
        let l = self.history.len();
        if n as usize > l
        {
            let err_msg = format!(
                "Asked for {} undo{}, but only {} turn{} {} been played on this board.",
                n,
                if n == 1 { "" } else { "s" },
                l,
                if l == 1 { "" } else { "s" },
                if l == 1 { "has" } else { "have" }
            );
            return Err(Error::new(Kind::TooManyUndos, err_msg));
        }

        for _ in 0..n
        {
            if let Err(err) = self.undo_one()
            {
                let gamestr: GameString = GameString::from(&*self);
                let base = Error::new(Kind::InternalError, format!("Failed to undo last move on GameString '{gamestr}'."));
                return Err(Error::holy_shit(err.chain(base)));
            }
        }

        Ok(self.zobrist.get())
    }

    /// Undoes the last move, if possible.
    pub fn undo_one(&mut self) -> Result<ZobristHash>
    {
        let Some(entry) = self.history.prev()
        else
        {
            return Err(Error::new(Kind::InternalError, "No move to undo.".into()));
        };

        // This function only nominally does error checking. Because we are undoing a move that previously passed a
        // Board::check() in the play step, we skip error checking and use the unchecked versions of insert and remove.
        // If something goes wrong, that is *concerning*.

        match entry.mv
        {
            // Very simple - just pull the piece out entirely, because the patch data is irrelevant.
            | Move::Place(piece, _) =>
            {
                self.remove_unchecked(&piece);
            }
            // This is a bit more involved, because we need to update the data at the from-hex.
            | Move::Move(piece, _) =>
            {
                self.remove_unchecked(&piece);

                let hex = entry.patch.unwrap().from.unwrap();
                self.insert_unchecked(&piece, hex);
            }
            // Absolutely nothing happened to the board state on this pass, except that we should set the immune hex.
            | Move::Pass =>
            {}
        };

        // Recalculate the pins.
        self.pinned = self.field.find_pins();

        // Find the last hex moved to (which might be None) and reset the immunity and stun states.
        self.undo_immune()?;
        self.undo_stun()?;

        // Flip the player.
        self.zobrist.prev();

        // Now that the look-backwards steps are done, fix the history.
        self.history.undo();

        Ok(self.zobrist.get())
    }

    /// Gets the key corresponding to this board.
    pub fn zobrist(&self) -> ZobristHash
    {
        self.zobrist.get()
    }
}

/// Private implementation for this board.
impl Board
{
    /// Ensures a piece can be inserted.
    fn can_insert(&self, piece: &Piece, hex: Hex) -> Result<()>
    {
        let axial = Axial::from(hex);
        let base = Error::new(Kind::LogicError, format!("Cannot insert {} at hex {}.", piece, axial));

        if self.placed(piece)
        {
            let at = self.pieces[piece.index() as usize].unwrap();
            let err = Error::new(
                Kind::InvalidState,
                format!("Piece {} is already in the hive at hex {}.", piece, Axial::from(at)),
            );
            return Err(err.chain(base));
        }

        if self.stacks[hex as usize].full()
        {
            let err = Error::new(Kind::InvalidState, format!("The target stack at hex {} is full.", axial));
            return Err(err.chain(base));
        }

        Ok(())
    }

    /// Ensures a piece can be moved in this manner.
    ///
    /// A piece can be moved if:
    ///
    /// 1. it is already in the Hive;
    /// 2. it is the top bug in its stack;
    /// 3. it is not stunned; and
    /// 4. it satisfies movement rules for its kind.
    fn can_move(&self, piece: &Piece, hex: Hex) -> Result<()>
    {
        let base = Error::new(Kind::InvalidMove, format!("Cannot move {} to {}.", piece, Axial::from(hex)));
        (|| {
            self.ensure_pieces_can_move()?;
            self.ensure_placed(piece)?;
            self.ensure_on_top(piece)?;
            self.ensure_active(piece)
        })()
        .map_err(|err: Error| err.chain(base.clone()))?;

        // Check that we can move like the piece we are from the source hex to the destination hex.
        // If not, we need to try a pillbug throw, but we can't rely on Board::is_pillbug_move(),
        // because it does not mark self-player moves as pillbug moves, and we might have one.

        let from = self.pieces[piece.index() as usize].unwrap();

        self.ensure_one_hive(piece).map_err(|err| err.chain(base.clone()))?;
        self.check_motion(piece, hex)
            .or_else(|mv_err| self.check_throw(from, hex).map_err(|err| err.chain(mv_err.chain(base))))
    }

    /// Ensures a piece can be placed into the Hive.
    ///
    /// A piece can be placed if:
    ///
    /// 1. it is in the pouch;
    /// 2. there is no bug of the same kind with a lower discriminator;
    /// 3. its target hex is unoccupied;
    /// 4. it has at least one friendly neighbour; and
    /// 5. it has no uncovered opposing neighbours.
    fn can_place(&self, piece: &Piece, hex: Hex) -> Result<()>
    {
        let axial = Axial::from(hex);
        let base = Error::new(Kind::InvalidMove, format!("Cannot place {} at hex {}.", piece, axial));
        (|| {
            // Ensure the choice of piece is valid.
            self.ensure_queen_placement(piece)?;
            self.ensure_correct_player(piece)?;
            self.ensure_unplaced(piece)?;
            self.ensure_lowest_discriminator(piece)?;

            // Ensure the choice of destination is valid.
            self.ensure_no_stack(hex)?;
            self.ensure_drop(piece, hex)
        })()
        .map_err(|err: Error| err.chain(base.clone()))?;

        Ok(())
    }

    #[allow(unused)]
    /// Ensures a piece can be removed.
    fn can_remove(&self, piece: &Piece) -> Result<()>
    {
        let base = Error::new(Kind::LogicError, format!("Cannot remove {} from the Hive.", piece));

        self.ensure_placed(piece).map_err(|err| err.chain(base.clone()))?;
        self.ensure_on_top(piece).map_err(|err| err.chain(base.clone()))?;
        self.ensure_one_hive(piece).map_err(|err| err.chain(base))?;

        Ok(())
    }

    /// Determines whether or not the given bug can act as a Pillbug this turn.
    pub(crate) fn can_throw_another(&self, piece: &Piece) -> bool
    {
        if !self.placed(piece)
            || self
                .stunned
                .map(|hex| hex == self.pieces[piece.index() as usize].unwrap())
                .unwrap_or(false)
            || piece.player != self.to_move()
        {
            false
        }
        else if piece.kind == Bug::Pillbug
        {
            true
        }
        else if piece.kind == Bug::Mosquito
        {
            let hex = self.pieces[piece.index() as usize].unwrap();
            self.pieces_neighbouring(hex).iter().any(|piece| piece.kind == Bug::Pillbug)
        }
        else
        {
            false
        }
    }

    /// Returns an error for when a movement check fails while acting as a particular bug.
    fn failed_as(&self, kind: Bug) -> Error
    {
        Error::new(Kind::LogicError, format!("This is not a valid {} move.", kind.long()))
    }

    #[allow(unused)]
    /// Inserts a piece into the Hive, updating the hash.
    ///
    /// A piece can only be inserted into the hive if:
    /// 1. it is not already in the Hive;
    /// 2. there is space at the target stack.
    fn insert(&mut self, piece: &Piece, hex: Hex) -> Result<()>
    {
        self.can_insert(piece, hex)?;
        self.insert_unchecked(piece, hex);
        Ok(())
    }

    /// Inserts a piece into the hive unchecked. Assumes [Self::can_insert()].
    fn insert_unchecked(&mut self, piece: &Piece, hex: Hex)
    {
        // Update the bag.
        self.pouch.take(piece.player, piece.kind);

        // Update the references.
        self.pieces[piece.index() as usize] = Some(hex);
        self.field.push(hex);

        // Put the token on the stack.
        let token: Token = (*piece).into();
        self.stacks[hex as usize].push(token);

        // Update the Zobrist hash.
        let height = self.stacks[hex as usize].height();
        self.zobrist.hash(piece, hex, height);
    }

    /// Translate a move into a patch, so we can supplement the history.
    fn patch_from(&self, mv: &Move) -> Option<Patch>
    {
        match mv
        {
            | Move::Pass => None,
            | Move::Move(piece, nextto) =>
            {
                let piece = *piece;
                let from = self.history.last_hex(piece);
                let to = self.resolve(&Some(*nextto));
                Some(Patch { piece, from, to })
            }
            | Move::Place(piece, nextto) =>
            {
                let piece = *piece;
                let from = self.history.last_hex(piece);
                let to = self.resolve(nextto);
                Some(Patch { piece, from, to })
            }
        }
    }

    /// Plays the move onto the board. Assumes Board::check().
    pub(crate) fn play_unchecked(&mut self, mv: &Move) -> ZobristHash
    {
        let entry = Entry {
            mv:           *mv,
            patch:        self.patch_from(mv),
            prev_stunned: self.stunned,
        };

        match mv
        {
            | Move::Place(piece, nextto) =>
            {
                // Put the piece in the Hive by taking it out of the bag.
                let hex = self.resolve(nextto);
                self.insert_unchecked(piece, hex);

                // It was the last piece moved/played, so it is immune to the Pillbug next turn.
                self.set_immune(Some(hex));
                self.set_stun(None);
            }
            | Move::Move(piece, nextto) =>
            {
                // Remove the piece from where it currently is.
                self.remove_unchecked(piece);

                // Insert it into its new location.
                let hex = self.resolve(&Some(*nextto));
                self.insert_unchecked(piece, hex);

                // It is now immune to the Pillbug on the next turn.
                self.set_immune(Some(hex));
                self.set_stun(Some(hex));
            }
            | Move::Pass =>
            {
                // No piece is immune.
                self.set_immune(None);
                self.set_stun(None);
            }
        };

        // Recalculate the pins.
        self.pinned = self.field.find_pins();

        // Update the history.
        self.history.play(entry);

        // Flip the player to move.
        self.zobrist.next();

        self.zobrist.get()
    }

    /// Determines if the target hex is isolated when removing the given piece.
    fn reachable(&self, piece: &Piece, to: Hex) -> bool
    {
        self.neighbours(to).into_iter().filter(|adj| *adj != *piece).count() > 0
    }

    #[allow(unused)]
    /// Removes a piece from the Hive, putting the piece back into the bag and updating the hash.
    fn remove(&mut self, piece: &Piece) -> Result<()>
    {
        self.can_remove(piece)?;
        self.remove_unchecked(piece);
        Ok(())
    }

    /// Removes a piece from the hive unchecked. Assumes Board::can_remove().
    fn remove_unchecked(&mut self, piece: &Piece)
    {
        let hex = self.pieces[piece.index() as usize].unwrap();
        let height = self.stacks[hex as usize].height();

        // Update the Zobrist key - remove this piece from its current location and height.
        self.zobrist.hash(piece, hex, height);

        // Remove the token from its stack.
        self.stacks.get_mut(hex as usize).unwrap().pop();

        // Remove all unshared references to its hex.
        self.pieces[piece.index() as usize] = None;
        self.field.pop(hex);

        // Put the piece back into the bag.
        self.pouch.put(*piece);
    }

    /// Resolves a reference from a move into a real grid coordinate.
    fn resolve(&self, nextto: &Option<NextTo>) -> Hex
    {
        match nextto
        {
            | Some(nextto) =>
            {
                let NextTo { piece, direction } = nextto;
                let dest = self.pieces[piece.index() as usize].expect(format!("Reference piece {} not in hive?", piece).as_str());
                match direction
                {
                    | Some(d) => dest + *d,
                    | None => dest,
                }
            }
            // The first placement of the game resolves to the origin hex.
            | None => hex::consts::ROOT,
        }
    }

    /// Sets the immunity hex.
    fn set_immune(&mut self, hex: Option<Hex>)
    {
        self.immune = hex;
        self.zobrist.last(self.immune);
    }

    // Sets the stunned hex.
    fn set_stun(&mut self, hex: Option<Hex>)
    {
        self.stunned = hex;
        self.zobrist.stun(self.stunned);
    }

    /// Sets the immune hex to the last destination.
    fn undo_immune(&mut self) -> Result<()>
    {
        let Some(entry) = self.history.prev()
        else
        {
            return Err(Error::new(
                Kind::InternalError,
                "Could not undo immunity because there is no previous move.".into(),
            ));
        };

        // Update the immune hex and the Zobrist hash.
        self.set_immune(entry.patch.map(|patch| patch.to));

        Ok(())
    }

    // Sets the stunned hex to the last destination, if the last move was a Pillbug throw.
    fn undo_stun(&mut self) -> Result<()>
    {
        let Some(entry) = self.history.prev()
        else
        {
            return Err(Error::new(
                Kind::InternalError,
                "Could not undo stun because there is no previous move.".into(),
            ));
        };

        // Update the stunned hex.
        self.set_stun(entry.prev_stunned);

        Ok(())
    }
}
