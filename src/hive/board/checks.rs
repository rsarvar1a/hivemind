use crate::prelude::*;

/// Movement implementation per bug type for this board.
impl Board
{
    /// Checks whether the bug can move as itself.
    pub(super) fn check_motion(&self, piece: &Piece, to: Hex) -> Result<()>
    {
        let from = self.pieces[piece.index() as usize].unwrap();
        self.check_motion_as(piece.kind, from, to)
    }

    /// Extracted behaviour of check_motion so that the mosquito can avoid making too many one-hive calls.
    pub(super) fn check_motion_as(&self, kind: Bug, from: Hex, to: Hex) -> Result<()>
    {
        let base = self.failed_as(kind);
        match kind
        {
            | Bug::Ant => self.check_ant(from, to),
            | Bug::Beetle => self.check_beetle(from, to),
            | Bug::Grasshopper => self.check_grasshopper(from, to),
            | Bug::Ladybug => self.check_ladybug(from, to),
            | Bug::Mosquito => self.check_mosquito(from, to),
            | Bug::Pillbug => self.check_pillbug(from, to),
            | Bug::Queen => self.check_queen(from, to),
            | Bug::Spider => self.check_spider(from, to),
        }
        .map_err(|err| err.chain(base))
    }

    /// Whether or not this movement occurred due to the actions of a Pillbug.
    pub(super) fn check_throw(&self, from: Hex, to: Hex) -> Result<()>
    {
        let base = Error::new(Kind::LogicError, "This movement was not caused by a Pillbug ability.".into());

        // Ensure the target can be thrown.

        if self.immune.map(|hex| hex == from).unwrap_or(false)
        {
            let axial_f = Axial::from(from);
            let err = Error::new(
                Kind::ImmuneToPillbug,
                format!("Hex {} is immune to the Pillbug ability this turn.", axial_f),
            );
            return Err(err.chain(base));
        }

        // Find which neighbour threw the pillbug.

        if !self
            .pieces_neighbouring(from)
            .into_iter()
            .filter(|piece| self.can_throw_another(piece))
            .map(|piece| self.check_throw_via(from, piece, to))
            .any(|r| r.is_ok())
        {
            let err = Error::new(Kind::LogicError, "None of this piece's neighbours can throw it.".into());
            return Err(err.chain(base));
        }

        Ok(())
    }

    /// Whether or not the movement can be interpreted as a throw if the given piece is doing the throwing.
    pub(super) fn check_throw_via(&self, from: Hex, via: Piece, to: Hex) -> Result<()>
    {
        let base = Error::new(Kind::InvalidMove, format!("Piece {} cannot execute this throw.", via));
        let intermediate = self.pieces[via.index() as usize].unwrap();
        self.ensure_ground_movement(from, to)
            .and_then(|_| self.ensure_crawl(from, intermediate, false))
            .and_then(|_| self.ensure_crawl(intermediate, to, true))
            .map_err(|err| err.chain(base))
    }

    #[inline]
    /// Ensures that a bug can crawl one hex.
    pub(super) fn ensure_crawl(&self, from: Hex, to: Hex, ghosting: bool) -> Result<()>
    {
        self.field
            .ensure_constant_contact(from, to, ghosting)
            .and_then(|_| self.field.ensure_freedom_to_move(from, to, ghosting))
    }

    #[inline]
    /// Ensures the movement both starts and ends on the ground, but makes no other guarantees.
    pub(super) fn ensure_ground_movement(&self, from: Hex, to: Hex) -> Result<()>
    {
        let base = Error::new(Kind::LogicError, "This movement is required to start and end on the ground.".into());

        let height_f = self.field.height(from).unwrap_or(0);
        let height_t = self.field.height(to).map(|height| height + 1).unwrap_or(1);

        if height_f > 1
        {
            let err = Error::new(Kind::LogicError, format!("Starting stack is {} bugs tall.", height_f));
            return Err(err.chain(base));
        }

        if height_t > 1
        {
            let err = Error::new(Kind::LogicError, format!("Ending stack height would be {}.", height_t));
            return Err(err.chain(base));
        }

        Ok(())
    }
}

impl Board
{
    #[inline]
    /// Whether or not this movement is valid as an Ant move.
    fn check_ant(&self, from: Hex, to: Hex) -> Result<()>
    {
        self.ensure_ground_movement(from, to)?;
        self.field.ensure_perimeter_crawl(from, to, None)
    }

    #[inline]
    /// Whether or not this movement is valid as a Beetle move.
    fn check_beetle(&self, from: Hex, to: Hex) -> Result<()>
    {
        self.ensure_crawl(from, to, false)
    }

    #[inline]
    /// Whether or not this movement is valid as a Grasshopper move.
    fn check_grasshopper(&self, from: Hex, to: Hex) -> Result<()>
    {
        #[inline]
        fn grasshopper_inner(board: &Board, from: Hex, to: Hex) -> Result<()>
        {
            // Test every possible jumping direction, which is every direction that has a neighbour.
            // It's a damn shame that our board wraps, otherwise we could just solve for the correct direction.

            if Direction::all()
                .into_iter()
                .filter(|direction| board.occupied(from + *direction))
                .any(|direction| {
                    let mut hex = from;
                    while board.occupied(hex)
                    {
                        hex = hex + direction;
                    }
                    hex == to
                })
            {
                Ok(())
            }
            else
            {
                Err(Error::new(Kind::LogicError, "Could not complete this jump in any direction.".into()))
            }
        }

        self.ensure_ground_movement(from, to)?;
        grasshopper_inner(self, from, to)
    }

    #[inline]
    /// Whether or not this movement is valid as a Ladybug move.
    fn check_ladybug(&self, from: Hex, to: Hex) -> Result<()>
    {
        #[inline]
        fn ladybug_inner(board: &Board, from: Hex, to: Hex) -> Result<()>
        {
            if board
                .field
                .neighbours(from)
                .into_iter()
                // Use the first movement to get onto the hive by only selecting in-hive neighbours.
                .filter_map(|onto| board.ensure_crawl(from, onto, false).map(|_| (onto, board.field.neighbours(onto))).ok())
                .flat_map(|(onto, neighbours)| neighbours.into_iter().map(move |h| (onto, h)))
                // Use the second movement to get to another hex on top of the Hive.
                // We explicitly filter out the starting hex, because we never removed its stack.
                .filter(|(.., ontop)| *ontop != from)
                .filter_map(|(onto, ontop)| board.ensure_crawl(onto, ontop, true).map(|_| ontop).ok())
                // If any of the 2nd-spot hexes drops down to the destination hex, which is not in the hive, we succeeded.
                .any(|ontop| board.ensure_crawl(ontop, to, true).is_ok())
            {
                Ok(())
            }
            else
            {
                Err(Error::new(
                    Kind::LogicError,
                    "Conducted an exhaustive search for paths, but failed.".into(),
                ))
            }
        }

        self.ensure_ground_movement(from, to)?;
        ladybug_inner(self, from, to)
    }

    #[inline]
    /// Whether or not this movement is valid as a Mosquito move.
    fn check_mosquito(&self, from: Hex, to: Hex) -> Result<()>
    {
        #[inline]
        fn mosquito_inner(board: &Board, from: Hex, to: Hex) -> Result<()>
        {
            let height = board.field.height(from).unwrap();
            if height > 1
            {
                // Must move as a beetle on the hive.
                board.check_motion_as(Bug::Beetle, from, to)
            }
            else if board
                .pieces_neighbouring(from)
                .iter()
                .filter(|piece| piece.kind != Bug::Mosquito)
                .any(|piece| board.check_motion_as(piece.kind, from, to).is_ok())
            {
                Ok(())
            }
            else
            {
                Err(Error::new(Kind::LogicError, "Could not move as any neighbouring bug.".into()))
            }
        }

        mosquito_inner(self, from, to)
    }

    #[inline]
    /// Whether or not this movement is valid as a Pillbug move.
    fn check_pillbug(&self, from: Hex, to: Hex) -> Result<()>
    {
        self.ensure_ground_movement(from, to)?;
        self.ensure_crawl(from, to, false)
    }

    #[inline]
    /// Whether or not this movement is valid as a Queen move.
    fn check_queen(&self, from: Hex, to: Hex) -> Result<()>
    {
        self.ensure_ground_movement(from, to)?;
        self.ensure_crawl(from, to, false)
    }

    #[inline]
    /// Whether or not this movement is valid as a Spider move.
    fn check_spider(&self, from: Hex, to: Hex) -> Result<()>
    {
        self.ensure_ground_movement(from, to)?;
        self.field.ensure_perimeter_crawl(from, to, Some(3))
    }
}
