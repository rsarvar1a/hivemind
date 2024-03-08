use std::collections::HashSet;

use crate::prelude::*;

impl Board
{
    /// Generates all valid moves in the position, not including Pass.
    pub fn generate_moves(&self, standard_position: bool) -> Vec<Move>
    {
        let mut moves: Vec<Move> = self.generate_non_throws(standard_position);
        self.generate_throws_into(&mut moves);

        // {
        //     let mut capture = None;
        //     for mv in &moves
        //     {
        //         let check = self.check(&mv);
        //         if let Err(err) = check
        //         {
        //             let err_here = err.chain(Error::new(Kind::InvalidMove, format!("Move {} is invalid.", mv)));
        //             capture = match capture
        //             {
        //                 | Some(c) => Some(err_here.chain(c)),
        //                 | None => Some(err_here),
        //             };
        //         }
        //     }
        //     if let Some(err) = capture
        //     {
        //         let with_gamestring = Error::new(
        //             Kind::LogicError,
        //             format!("Generated invalid moves in position {}.", GameString::from(self)),
        //         );
        //         let base = Error::holy_shit(err.chain(with_gamestring));
        //         panic!("{}", base);
        //     }
        // }

        moves
    }

    /// Only generates tactical moves for quiescence search. 
    pub fn generate_tactical_moves(&self) -> Vec<Move>
    {
        // Don't waste time here in the opening.
        if self.turn() < 8 
        {
            return Vec::new();
        }

        let mut moves: Vec<Move> = Vec::new();
        let past = self.history.get_past();

        // Check our last placement to see what its extensions are.
        // This is because placing a piece is a loss of pinning tempo on the board,
        // and should yield power elsewhere.
        'attack: {
            if let Move::Place(piece, _) = past[past.len() - 2].mv
            {
                let entry = past[past.len() - 2];
                let destination = entry.patch.unwrap().to;

                // Check if this is a direct drop. A direct drop implies we are covering
                // the queen with a piece of our own, so we can't just check the tops of
                // the neighbouring stacks.

                if let Some(enemy_queen_location) = self.queen(self.to_move().flip())
                {
                    let mut direct_drop = false;
                    for direction in Direction::all()
                    {
                        let reference = destination + direction;
                        if reference == enemy_queen_location
                        {
                            direct_drop = true;
                            break;
                        }
                    }

                    if direct_drop
                    {
                        break 'attack;
                    }
                }

                // Otherwise, this is too quiet, so we should check extensions.

                self.generate_moves_for(&piece, &mut moves);
                return moves;
            }
        }

        // If we didn't just make a quiet move, but the opposing player did, generate 
        // a full subtree to allow the opposing player the opportunity to find extensions.
        'defense: {
            if let Move::Place(..) = past[past.len() - 1].mv
            {
                let entry = past[past.len() - 1];
                let destination = entry.patch.unwrap().to;

                // Once again, check if this is a direct drop.

                if let Some(friendly_queen_location) = self.queen(self.to_move())
                {
                    let mut direct_drop = false;
                    for direction in Direction::all()
                    {
                        let reference = destination + direction;
                        if reference == friendly_queen_location
                        {
                            direct_drop = true;
                            break;
                        }
                    }

                    if direct_drop
                    {
                        break 'defense;
                    }
                }

                // Otherwise, generate a full subtree.

                moves = self.generate_moves(false);
            }
        }

        moves
    }
}

impl Board
{
    /// Generates true moves for the player to move (not including throws).
    pub(super) fn generate_moves_into(&self, moves: &mut Vec<Move>)
    {
        let to_move = self.to_move();
        if self.queen(to_move).is_none()
        {
            // Can't move pieces until the queen is in the hive.
            return;
        }

        let movable = self
            .pieces
            .iter()
            // Map each hex to the piece index it corresponds to.
            .enumerate()
            // Get the pieces from the indices.
            .map(|(i, on_board)| (Piece::from(i as u8), on_board))
            // Drop the pieces that are pinned, and ensure they're not stunned.
            .filter_map(|(piece, on_board)| on_board.map(|loc| (!self.is_pinned(&piece) && self.stunned != Some(loc)).then_some(piece)).unwrap_or(None))
            // Only move pieces owned by the current player.
            .filter(|piece| piece.player == to_move)
            // Take uniques.
            .collect::<HashSet<Piece>>();

        movable.iter().for_each(|piece| {
            self.generate_moves_for(piece, moves);
        });
    }

    /// Generates placements for the player to move.
    pub(super) fn generate_placements_into(&self, standard_position: bool, moves: &mut Vec<Move>)
    {
        let to_move = self.to_move();
        let deploys = self.hexes_for_placements(standard_position);
        let reserve = if self.queen(to_move).is_none() && self.turn() >= 6
        {
            // The queen must be placed before the end of the fourth turn.
            HashSet::from([Piece {
                player: to_move,
                kind:   Bug::Queen,
                num:    1,
            }])
        }
        else
        {
            Bug::all()
                .iter()
                // Get the next piece of each bug type, which handily drops bugs not included in the pouch due to expansion settings.
                .filter_map(|b| self.pouch.next(to_move, *b))
                // Drop the queen if we are on the first turn..
                .filter(|p| self.turn() >= 2 || p.kind != Bug::Queen)
                // Take uniques.
                .collect::<HashSet<Piece>>()
        };

        reserve.iter().for_each(|piece| {
            deploys.iter().for_each(|hex| {
                moves.push(Move::Place(*piece, self.reference(piece, *hex)));
            });
        });
    }

    pub(super) fn generate_throws_into(&self, moves: &mut Vec<Move>)
    {
        let to_move = self.to_move();
        if self.queen(to_move).is_none()
        {
            // Can't throw bugs until the queen is in the hive.
            return;
        }

        [
            Piece {
                player: to_move,
                kind:   Bug::Mosquito,
                num:    1,
            },
            Piece {
                player: to_move,
                kind:   Bug::Pillbug,
                num:    1,
            },
        ]
        .into_iter()
        // Ensure the conditions are met (e.g. the mosquito must neighbour a pillbug).
        .filter(|piece| self.can_throw_another(piece))
        // Insert its throws into the movelist.
        .for_each(|piece| {
            let intermediate = self.pieces[piece.index() as usize].unwrap();
            let neighbours = hex::neighbours(intermediate);

            // Iterate over the cartesian product of the throwing piece's neighbours.
            itertools::iproduct!(neighbours.iter(), neighbours.iter())
                // Check if the source piece is pinned or not.
                .filter(|(from, _t)| self.top(**from).map(|p| self.ensure_one_hive(&p).is_ok()).unwrap_or(false))
                // Check if the piece can throw a bug from the source to the destination tile.
                // This also checks the immunity state.
                .filter(|(from, to)| self.check_throw(**from, **to).is_ok())
                // Then construct the movement by figuring out which piece was thrown.
                .for_each(|(from, to)| {
                    let moving = self.top(*from).unwrap();
                    let reference = self.reference(&moving, *to);
                    moves.push(Move::Move(moving, reference.unwrap()));
                });
        });
    }
}

impl Board
{
    /// Determines whether the bug on top is friendly.
    fn friendly(&self, hex: Hex) -> bool
    {
        let to_move = self.to_move();
        let on_top: Option<Piece> = self.stacks[hex as usize].top().into();
        on_top.map(|piece| piece.player == to_move).unwrap_or(false)
    }

    /// Returns all the hexes in which the current player can drop a piece.
    fn hexes_for_placements(&self, standard_position: bool) -> HashSet<Hex>
    {
        if self.turn() == 0
        {
            HashSet::from([hex::consts::ROOT])
        }
        else if self.turn() == 1
        {
            if standard_position
            {
                HashSet::from([hex::consts::ROOT + Direction::East])
            }
            else 
            {
                HashSet::from(hex::neighbours(hex::consts::ROOT))    
            }
        }
        else
        {
            let to_move = self.to_move();
            self.pieces
                .iter()
                // Get the occupied hexes with a friendly bug on top.
                .filter_map(|h| h.and_then(|hex| self.friendly(hex).then_some(hex)))
                // Get all of their neighbours.
                .flat_map(|h| hex::neighbours(h))
                // Remove any neighbour that's occupied or that has an unfriendly neighbour.
                .filter(|h| !self.occupied(*h) && !self.neighbours(*h).iter().any(|p| p.player != to_move))
                // Take uniques.
                .collect()
        }
    }

    /// "Unresolves" a hex, giving a reference that can be encoded into a movestring.
    fn reference(&self, moving: &Piece, hex: Hex) -> Option<NextTo>
    {
        // Check for climbing.
        if let Some(stack) = self.top(hex)
        {
            return Some(NextTo { piece: stack, direction: None });
        }

        // Return the best directional marker in some sort of clockwise order.
        for dir in Direction::all()
        {
            let loc = hex - dir;
            if ! self.occupied(loc)
            {
                continue;
            }

            let piece = self.top(loc).unwrap();
            if piece == *moving
            {
                continue;
            }

            return Some(NextTo { piece, direction: Some(dir) });
        }

        None
    }
}

impl Board
{
    /// Generates ground crawls for the given piece.
    fn generate_ground_crawls(&self, piece: &Piece, moves: &mut Vec<Move>)
    {
        let from = self.pieces[piece.index() as usize].unwrap();
        hex::neighbours(from)
            .iter()
            // Drop impossible destinations.
            .filter(|to| !self.occupied(**to))
            // Drop destinations that are isolated.
            .filter(|to| self.reachable(piece, **to))
            // Drop movements that don't end at ground level.
            .filter(|to| self.ensure_ground_movement(from, **to).is_ok())
            // Drop movements that violate freedom to move and constant contact.
            .filter(|to| self.ensure_crawl(from, **to, false).is_ok())
            .for_each(|to| {
                let reference = self.reference(piece, *to).unwrap();
                moves.push(Move::Move(*piece, reference));
            });
    }

    /// Finds all of the ways this piece can move.
    pub(super) fn generate_moves_for(&self, piece: &Piece, moves: &mut Vec<Move>)
    {
        // This is a convenient way to enter the generation logic while allowing the Mosquito an easy way to recurse on its neighbours.
        self.generate_moves_for_kind(piece, piece.kind, moves)
    }

    /// Generates all of the moves for the given piece as if it was acting as the given bug type.
    fn generate_moves_for_kind(&self, piece: &Piece, kind: Bug, moves: &mut Vec<Move>)
    {
        match kind
        {
            | Bug::Ant => self.generate_ant(piece, moves),
            | Bug::Beetle => self.generate_beetle(piece, moves),
            | Bug::Grasshopper => self.generate_grasshopper(piece, moves),
            | Bug::Ladybug => self.generate_ladybug(piece, moves),
            | Bug::Mosquito => self.generate_mosquito(piece, moves),
            | Bug::Pillbug => self.generate_pillbug(piece, moves),
            | Bug::Queen => self.generate_queen(piece, moves),
            | Bug::Spider => self.generate_spider(piece, moves),
        }
    }

    /// Generates ant moves for the given piece.
    fn generate_ant(&self, piece: &Piece, moves: &mut Vec<Move>)
    {
        let from = self.pieces[piece.index() as usize].unwrap();
        let reachable = self.field.find_crawls(from, None);

        reachable.iter()
            .filter(|to| **to != from)
            .for_each(|to| {
            let reference = self.reference(piece, *to).unwrap();
            moves.push(Move::Move(*piece, reference));
        });
    } 

    /// Generates beetle moves for the given piece.
    fn generate_beetle(&self, piece: &Piece, moves: &mut Vec<Move>)
    {
        let from = self.pieces[piece.index() as usize].unwrap();
        hex::neighbours(from)
            .iter()
            // Ensure destinations are neighboured.
            .filter(|to| self.reachable(piece, **to))
            // Drop movements that violate freedom to move and constant contact.
            .filter(|to| self.ensure_crawl(from, **to, false).is_ok())
            .for_each(|to| {
                let reference = self.reference(piece, *to).unwrap();
                moves.push(Move::Move(*piece, reference));
            });
    }

    /// Generates grasshopper jumps for the given piece.
    fn generate_grasshopper(&self, piece: &Piece, moves: &mut Vec<Move>)
    {
        let from = self.pieces[piece.index() as usize].unwrap();
        Direction::all().iter().for_each(|direction| {
            let mut to = from + *direction;
            
            // If there's no neighbour, don't start the jump.
            if ! self.occupied(to)
            {
                return;
            }

            // Keep jumping in the same direction until we land in an empty hex.
            while self.occupied(to)
            {
                to = to + *direction;
            }

            let reference = self.reference(piece, to).unwrap();
            moves.push(Move::Move(*piece, reference));
        });
    }

    /// Generates ladybug movements.
    fn generate_ladybug(&self, piece: &Piece, moves: &mut Vec<Move>)
    {
        let from = self.pieces[piece.index() as usize].unwrap();
        let to = self
            .field
            .neighbours(from)
            .into_iter()
            // Get onto the hive with the first movement, only selecting in-hive neighbours.
            .filter_map(|onto| self.ensure_crawl(from, onto, false).map(|_| (onto, self.field.neighbours(onto))).ok())
            // Get the path tuples.
            .flat_map(|(onto, neighbours)| neighbours.into_iter().map(move |h| (onto, h)))
            // Remove any that doubled back to the start hex.
            .filter(|(.., ontop)| *ontop != from)
            // Move from an oh-hive hex to another on-hive hex, and get that destination's neighbours.
            .filter_map(|(onto, ontop)| self.ensure_crawl(onto, ontop, true).map(|_| (onto, ontop, hex::neighbours(ontop))).ok())
            // Get the path tuples.
            .flat_map(|(onto, ontop, neighbours)| neighbours.into_iter().map(move |h| (onto, ontop, h)))
            // Remove any that doubled back to a previous hex.
            .filter(|(onto, _, to)| *to != from && *to != *onto)
            // Remove any that aren't ground-level.
            .filter(|(.., to)| self.ensure_ground_movement(from, *to).is_ok())
            // Drop down into that hex.
            .filter_map(|(_, ontop, to)| self.ensure_crawl(ontop, to, true).map(|_| to).ok())
            // Take uniques.
            .collect::<HashSet<Hex>>();

        to.iter().for_each(|to| {
            let reference = self.reference(piece, *to).unwrap();
            moves.push(Move::Move(*piece, reference));
        });
    }

    /// Recursively generates moves for the mosquito.
    fn generate_mosquito(&self, piece: &Piece, moves: &mut Vec<Move>)
    {
        let from = self.pieces[piece.index() as usize].unwrap();
        let height = self.stacks[from as usize].height();

        if height == 1
        {
            self.pieces_neighbouring(from)
                .iter()
                // The mosquito cannot move through the ability stolen by a neighbouring mosquito.
                .filter(|piece| piece.kind != Bug::Mosquito)
                // Generate moves for each piece as if the mosquito stole its type.
                .for_each(|moving_as| {
                    self.generate_moves_for_kind(piece, moving_as.kind, moves);
                });
        }
        else
        {
            self.generate_beetle(piece, moves);
        }
    }

    /// Generates standard crawls as if the piece was a Pillbug.
    fn generate_pillbug(&self, piece: &Piece, moves: &mut Vec<Move>)
    {
        self.generate_ground_crawls(piece, moves);
    }

    /// Generates standard crawls as if the piece was a Queen.
    fn generate_queen(&self, piece: &Piece, moves: &mut Vec<Move>)
    {
        self.generate_ground_crawls(piece, moves);
    }

    /// Generates spider moves for the given piece.
    fn generate_spider(&self, piece: &Piece, moves: &mut Vec<Move>)
    {
        let from = self.pieces[piece.index() as usize].unwrap();
        let reachable = self.field.find_crawls(from, Some(3));

        reachable.iter()
            .filter(|to| **to != from)
            .for_each(|to| {
            let reference = self.reference(piece, *to).unwrap();
            moves.push(Move::Move(*piece, reference));
        });
    }
}
