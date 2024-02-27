use crate::prelude::*;

impl Board
{
    #[inline]
    /// Ensures the piece is not stunned.
    pub(super) fn ensure_active(&self, piece: &Piece) -> Result<()>
    {
        let hex = self.pieces[piece.index() as usize].unwrap();
        if self.stunned.is_some() && self.stunned.unwrap() == hex
        {
            let err = Error::new(Kind::InvalidState, format!("Piece {} was stunned by a Pillbug.", piece));
            return Err(err);
        }
        Ok(())
    }

    #[inline]
    /// Ensures a played piece belongs to the player moving this turn.
    pub(super) fn ensure_correct_player(&self, piece: &Piece) -> Result<()>
    {
        let to_move = self.to_move();
        if piece.player != to_move
        {
            let err = Error::new(Kind::InvalidState, format!("Cannot place or directly move a {} bug on {}'s turn.", piece.player, to_move));
            return Err(err);
        }
        Ok(())
    }

    #[inline]
    /// Ensures the piece can be dropped here.
    pub(super) fn ensure_drop(&self, piece: &Piece, hex: Hex) -> Result<()>
    {
        let neighbours = self.neighbours(hex);

        if self.field.len() > 2
        {
            let Some(_) = neighbours.iter().find(|neighbour| neighbour.player == piece.player)
            else
            {
                let axial = Axial::from(hex);
                let err = Error::new(Kind::InvalidState, format!("Hex {} does not neighbour a friendly piece.", axial));
                return Err(err);
            };

            if let Some(offending_bug) = neighbours.iter().find(|neighbour| neighbour.player != piece.player)
            {
                let axial = Axial::from(hex);
                let err = Error::new(Kind::InvalidState, format!("Hex {} neighbours opposing piece {}.", axial, offending_bug));
                return Err(err);
            }
            Ok(())
        }
        else if self.field.len() == 1 && !hex::neighbours(hex::consts::ROOT).contains(&hex)
        {
            let err = Error::new(Kind::InvalidState, "Must neighbour the starting piece.".into());
            Err(err)
        }
        else
        {
            Ok(())
        }
    }

    #[inline]
    /// Ensures the piece being played has a lower discriminator than any other unplayed piece of the same bug type.
    pub(super) fn ensure_lowest_discriminator(&self, piece: &Piece) -> Result<()>
    {
        let Some(real_num) = self.pouch.peek(piece.player, piece.kind)
        else
        {
            let err = Error::new(Kind::InvalidState, format!("There are no more {}s to play.", piece.kind.long()));
            return Err(err);
        };

        if real_num != piece.num
        {
            let err = Error::new(
                Kind::MismatchError,
                format!(
                    "The next {} to place is {}, but tried to place {}.",
                    piece.kind.long(),
                    Piece { num: real_num, ..*piece },
                    piece
                ),
            );
            return Err(err);
        }
        Ok(())
    }

    #[inline]
    /// Ensures the destination has no stack, or tells us what is on top of that stack.
    pub(super) fn ensure_no_stack(&self, hex: Hex) -> Result<()>
    {
        let stack = self.stacks[hex as usize];
        if stack.height() != 0
        {
            let axial = Axial::from(hex);
            let piece_at_stack = Into::<Option<Piece>>::into(stack.top()).unwrap();
            let err = Error::new(
                Kind::InvalidState,
                format!("Hex {} is already occupied by the stack ending in {}.", axial, piece_at_stack),
            );
            return Err(err);
        }
        Ok(())
    }

    #[inline]
    /// Ensures the piece is on top, provided it is in the hive.
    pub(super) fn ensure_on_top(&self, piece: &Piece) -> Result<()>
    {
        if !self.on_top(piece)
        {
            Err(Error::new(Kind::InvalidState, format!("Piece {} is not on the top of its stack.", piece)))
        }
        else
        {
            Ok(())
        }
    }

    #[inline]
    /// Ensures the queen is already in the Hive.
    pub(super) fn ensure_pieces_can_move(&self) -> Result<()>
    {
        let turn: Turn = self.turn().into();
        let queen = Piece {
            player: turn.player,
            kind:   Bug::Queen,
            num:    1,
        };
        if self.pieces[queen.index() as usize].is_none()
        {
            let err = Error::new(Kind::InvalidState, "Pieces cannot move before the queen is placed.".into());
            return Err(err);
        }
        Ok(())
    }

    #[inline]
    /// Ensures the piece is placed.
    pub(super) fn ensure_placed(&self, piece: &Piece) -> Result<()>
    {
        if !self.placed(piece)
        {
            Err(Error::new(Kind::InvalidState, format!("Piece {} is not in the Hive.", piece)))
        }
        else
        {
            Ok(())
        }
    }

    #[inline]
    /// Ensures this placement follows the constraints on when a queen can be placed into the Hive.
    pub(super) fn ensure_queen_placement(&self, piece: &Piece) -> Result<()>
    {
        let turn: Turn = self.turn().into();
        if turn.turn == 1 && piece.kind == Bug::Queen
        {
            let err = Error::new(Kind::InvalidState, "The queen cannot be placed on the 1st turn.".into());
            return Err(err);
        };

        if turn.turn == 4 && piece.kind != Bug::Queen && self.queen(turn.player).is_none()
        {
            let err = Error::new(Kind::InvalidState, "The queen must be placed by the end of the 4th turn.".into());
            return Err(err);
        }
        Ok(())
    }

    #[inline]
    /// Ensures the piece is not in the hive.
    pub(super) fn ensure_unplaced(&self, piece: &Piece) -> Result<()>
    {
        if self.placed(piece)
        {
            let at = self.pieces[piece.index() as usize].unwrap();
            let err = Error::new(
                Kind::InvalidState,
                format!("Piece {} is already in the hive at hex {}.", piece, Axial::from(at)),
            );
            Err(err)
        }
        else
        {
            Ok(())
        }
    }
}
