use std::collections::{hash_map::Entry as MapEntry, HashMap, HashSet};

use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// A set of height-sensitive hexes useful for performing reachability calculations.
pub struct Field
{
    map: HashMap<Hex, u8>,
}

impl FromIterator<Hex> for Field
{
    fn from_iter<T: IntoIterator<Item = Hex>>(iter: T) -> Self
    {
        Field {
            map: iter.into_iter().map(|h| (h, 1)).collect(),
        }
    }
}

impl From<Field> for HashSet<Hex>
{
    fn from(value: Field) -> Self
    {
        value.map.keys().copied().collect()
    }
}

impl Field
{
    /// Determines whether this hex is inside the field.
    pub fn contains(&self, h: Hex) -> bool
    {
        self.map.contains_key(&h)
    }

    /// Ensures that the two hexes are neighbours, and returns their common neighbours.
    pub fn ensure_common_neighbours(&self, from: Hex, to: Hex) -> Result<(Hex, Hex)>
    {
        let Some((cw, ccw)) = hex::common_neighbours(from, to)
        else
        {
            let axial_f = Axial::from(from);
            let axial_t = Axial::from(to);
            return Err(Error::new(
                Kind::InvalidState,
                format!("Hex {} and hex {} are not neighbours.", axial_f, axial_t),
            ));
        };

        Ok((cw, ccw))
    }

    /// Ensures that a movement between two hexes satisfies the constant contact rule.
    ///
    /// This is true when the hexes are neighbours and:
    /// 1. a ground-level movement has a common neighbour; or
    /// 2. one hex is elevated.
    ///
    /// If the from-hex is not in the hive, the check assumes there is a piece at **ground level**.
    ///
    /// Passing ghosting=true tells it to assume there is a piece one higher in the stack instead.
    pub fn ensure_constant_contact(&self, from: Hex, to: Hex, ghosting: bool) -> Result<()>
    {
        let axial_f = Axial::from(from);
        let axial_t = Axial::from(to);

        let base = Error::new(
            Kind::ConstantContact,
            format!("Moving from hex {} to hex {} violates the constant contact principle.", axial_f, axial_t),
        );

        // Get the common neighbours between the two hexes to ensure they are neighbours.
        let (cw, ccw) = self.ensure_common_neighbours(from, to).map_err(|err| err.chain(base.clone()))?;

        // The height we are moving from is correct, because we haven't decremented the height at that stack yet.
        // The height of the destination should be incremented, because we would end up adding one if this movement were correct.
        let ghosting = if ghosting { 1 } else { 0 };
        let height_f = self.height(from).unwrap_or(1) + ghosting;
        let height_t = self.height(to).map(|h| h + 1).unwrap_or(1);

        if height_f.max(height_t) > 1
        {
            // Then the bug is always touching at least one bug, because either the from-stack or to-stack has a bug underneath and sharing an edge.
            Ok(())
        }
        else
        {
            // Otherwise, there is nothing underneath our feet, and we need a neighbour to the side.
            if !(self.contains(cw) || self.contains(ccw))
            {
                let err = Error::new(
                    Kind::InvalidState,
                    format!("Neither common neighbour, {} or {}, is in the hive.", cw, ccw),
                );
                Err(err.chain(base))
            }
            else
            {
                Ok(())
            }
        }
    }

    /// Ensures that a movement between two hexes satisfies the freedom to move rule.
    ///
    /// If the from-hex is not in the hive, the check assumes there is a piece at **ground level**.
    ///
    /// Passing ghosting=true tells it to assume there is a piece one higher in the stack instead.
    pub fn ensure_freedom_to_move(&self, from: Hex, to: Hex, ghosting: bool) -> Result<()>
    {
        let axial_f = Axial::from(from);
        let axial_t = Axial::from(to);

        let base = Error::new(
            Kind::FreedomToMove,
            format!("Moving from hex {} to hex {} violates the freedom to move principle.", axial_f, axial_t),
        );

        // Get the common neighbours between the two hexes to ensure they are neighbours.
        let (cw, ccw) = self.ensure_common_neighbours(from, to).map_err(|err| err.chain(base.clone()))?;

        if self.contains(cw) && self.contains(ccw)
        {
            let height_cw = self.height(cw).unwrap();
            let height_ccw = self.height(ccw).unwrap();

            let ghosting = if ghosting { 1 } else { 0 };
            let height_f = self.height(from).unwrap_or(1) + ghosting;
            let height_t = self.height(to).map(|h| h + 1).unwrap_or(1);

            let height_path = height_f.max(height_t);
            let height_gate = height_cw.min(height_ccw);

            if height_gate >= height_path
            {
                let err = Error::new(
                    Kind::InvalidState,
                    format!(
                        "Neighbouring hexes form a gate at least {} bugs tall, which gates the movement at height {}.",
                        height_gate, height_path
                    ),
                );
                Err(err.chain(base))
            }
            else
            {
                Ok(())
            }
        }
        else
        {
            Ok(())
        }
    }

    /// Ensures that a movement between two hexes satisfies the one hive principle.
    ///
    /// This is true if:
    /// 1. removing the from-hex results in a connected graph.
    pub fn ensure_one_hive(&self, piece: &Piece, from: Hex) -> Result<()>
    {
        if !self.contains(from)
        {
            // The field maintains the one hive principle as an invariant,
            // so if this hex is not in the hive, removing it would not break the invariant.
            // A good board implementation means this check is useless, since we would never
            // check the one hive principle on a move for a piece not in the hive, but it is
            // a really useful short-circuit.
            Ok(())
        }
        else if *self.map.get(&from).unwrap() > 1
        {
            // If the height of the stack is greater than 1, we can never break the principle
            // in this manner, like for beetle or mosquito-beetle moves.
            Ok(())
        }
        else if !self.find_pins().contains(&from)
        {
            Ok(())
        }
        else
        {
            let axial = Axial::from(from);
            Err(Error::new(
                Kind::OneHivePrinciple,
                format!("Piece {} started at hex {} and is pinned by the one hive principle.", piece, axial),
            ))
        }
    }

    /// Ensures an ant or spider move is possible for a given limit.
    pub fn ensure_perimeter_crawl(&self, from: Hex, to: Hex, distance: Option<u8>) -> Result<()>
    {
        if !self.find_crawls(from, distance).contains(&to)
        {
            if let Some(limit) = distance
            {
                let axial = Axial::from(to);
                Err(Error::new(
                    Kind::LogicError,
                    format!("Hex {} is not reachable in exactly {} steps.", axial, limit),
                ))
            }
            else
            {
                let axial = Axial::from(to);
                Err(Error::new(Kind::LogicError, format!("Hex {} is not reachable.", axial)))
            }
        }
        else
        {
            Ok(())
        }
    }

    /// Returns all ground hexes reachable by crawling some exact number of hexes.
    pub fn find_crawls(&self, from: Hex, distance: Option<u8>) -> HashSet<Hex>
    {
        let perimeter = self.perimeter(Some(from));

        if let Some(length) = distance
        {
            perimeter.exact_distance(from, length)
        }
        else
        {
            perimeter.reachable(from)
        }
    }

    /// Gets the height of the given hex in the field, if it exists.
    pub fn height(&self, h: Hex) -> Option<u8>
    {
        self.map.get(&h).copied()
    }

    /// Determines whether or not the hive is empty.
    pub fn is_empty(&self) -> bool
    {
        self.map.is_empty()
    }

    /// Determines whether the hex is locked behind a gate.
    pub fn is_gated(&self, hex: Hex) -> bool
    {
        self.neighbours(hex).len() >= 5
    }

    /// Gets the size of the field.
    pub fn len(&self) -> usize
    {
        self.map.len()
    }

    /// Returns the neighbours of the given hex that are present in the field.
    pub fn neighbours(&self, hex: Hex) -> HashSet<Hex>
    {
        hex::neighbours(hex).into_iter().filter(|hex| self.contains(*hex)).collect()
    }

    /// Returns the field consisting of the perimeter.
    pub fn perimeter(&self, as_if_without: Option<Hex>) -> Perimeter
    {
        let mut field = self.clone();
        if let Some(hex) = as_if_without
        {
            field.pop(hex);
        }

        let perim: Field = field
            .map
            .keys()
            .flat_map(|hex| hex::neighbours(*hex))
            .filter(|hex| !field.contains(*hex) && !field.is_gated(*hex))
            .collect();

        Perimeter(perim, field)
    }

    /// Removes a hex from the field.
    pub fn pop(&mut self, hex: Hex)
    {
        if let MapEntry::Occupied(mut o) = self.map.entry(hex)
        {
            if *o.get() == 1u8
            {
                o.remove_entry();
            }
            else
            {
                *o.get_mut() -= 1;
            }
        }
    }

    /// Adds a hex to the field.
    pub fn push(&mut self, hex: Hex)
    {
        *self.map.entry(hex).or_insert(0) += 1;
    }
}

// An implementation Tarjan's algorithm for finding articulation points.

#[derive(Clone, Copy, Debug, Default)]
struct HexStats
{
    num: u8,
    low: u8,
}

#[derive(Default)]
struct DescentRecord
{
    visited: HashMap<Hex, HexStats>,
    pinned:  HashSet<Hex>,
    count:   u8,
}

impl Field
{
    /// Returns all of the pinned hexes.
    pub fn find_pins(&self) -> HashSet<Hex>
    {
        if let Some(start) = self.map.keys().next()
        {
            let mut state = DescentRecord {
                count: 1,
                ..Default::default()
            };
            self.find_pins_recurse(*start, None, &mut state);
            state.pinned
        }
        else
        {
            HashSet::new()
        }
    }

    /// Recursively finds all cut vertices in the field.
    fn find_pins_recurse(&self, hex: Hex, parent: Option<Hex>, state: &mut DescentRecord)
    {
        state.visited.insert(
            hex,
            HexStats {
                num: state.count,
                low: state.count,
            },
        );
        state.count += 1;

        let mut children = 0;
        for neighbour in self.neighbours(hex)
        {
            let mut prev = state.visited.get(&hex).copied().unwrap();

            if let Some(par) = parent
            {
                if par == neighbour
                {
                    continue;
                }
            }

            if let Some(neighbour_stats) = state.visited.get(&neighbour)
            {
                prev.low = prev.low.min(neighbour_stats.num);
                state.visited.insert(hex, prev);
            }
            else
            {
                self.find_pins_recurse(neighbour, Some(hex), state);
                children += 1;

                let neighbour_stats = state.visited.get(&neighbour).copied().unwrap();
                prev.low = prev.low.min(neighbour_stats.low);
                state.visited.insert(hex, prev);

                if parent.is_some() && neighbour_stats.low >= prev.num
                {
                    state.pinned.insert(hex);
                }
            }
        }
        if parent.is_none() && children > 1
        {
            state.pinned.insert(hex);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A field that was specifically constructed from a perimeter.
///
/// The perimeter contains every hex that:
/// 1. is unoccupied;
/// 2. has at least one occupied neighbouring hex; and
/// 3. has fewer than 5 occupied neighbours.
///
/// In particular, condition 3. removes hexes that are inaccessible,
/// either because they are totally surrounded, or because they are
/// locked behind gates.
pub struct Perimeter(pub(super) Field, pub(super) Field);

impl From<Perimeter> for Field
{
    // Returns the field that generated this perimeter.
    fn from(value: Perimeter) -> Self
    {
        value.1
    }
}

impl From<Field> for Perimeter
{
    // Returns the perimeter of this field as if no hex was removed.
    fn from(value: Field) -> Self
    {
        value.perimeter(None)
    }
}

#[derive(Default)]
struct PathRecord
{
    visited:        HashSet<Hex>,
    reached:        HashSet<Hex>,
    starting_depth: u8,
    depth:          u8,
}

// An implementation of DFS for finding hexes reachable using paths of a particular length.

impl Perimeter
{
    /// Returns all hexes in the perimeter reachable using a non-backtracking path of the given length.
    pub fn exact_distance(&self, from: Hex, length: u8) -> HashSet<Hex>
    {
        log::trace!("Length {} DFS for {}:", length, Axial::from(from));

        if self.0.contains(from)
        {
            let mut state = PathRecord::default();
            state.visited.insert(from);
            state.starting_depth = length;
            state.depth = length;

            self.exact_distance_recurse(from, &mut state);
            state.reached
        }
        else
        {
            HashSet::new()
        }
    }

    /// Determines the set of all reachable hexes from the given starting hex.
    /// The starting hex is not reachable from itself, because no movement in Hive
    /// can involve a cyclic subpath.
    pub fn reachable(&self, from: Hex) -> HashSet<Hex>
    {
        log::trace!("Reachability DFS for {}:", Axial::from(from));

        if self.0.contains(from)
        {
            let mut state = PathRecord::default();
            self.reachable_recurse(from, &mut state);
            state.visited
        }
        else
        {
            HashSet::new()
        }
    }

    /// Recursively visits neighbours until we reach the desired path depth.
    fn exact_distance_recurse(&self, hex: Hex, state: &mut PathRecord)
    {
        if state.depth == 0
        {
            log::trace!("Path of exact length to {}.", Axial::from(hex));
            state.reached.insert(hex);
        }
        else
        {
            for neighbour in self.0.neighbours(hex)
            {
                if state.visited.contains(&neighbour)
                    || self.1.ensure_freedom_to_move(hex, neighbour, false).is_err()
                    || self.1.ensure_constant_contact(hex, neighbour, false).is_err()
                {
                    // The original field forms a gate here.
                    continue;
                }

                state.depth -= 1;
                state.visited.insert(neighbour);
                self.exact_distance_recurse(neighbour, state);
                state.visited.remove(&neighbour);
                state.depth += 1;
            }
        }
    }

    fn reachable_recurse(&self, from: Hex, state: &mut PathRecord)
    {
        state.visited.insert(from);
        log::trace!("Visited {}", Axial::from(from));

        for neighbour in self.0.neighbours(from)
        {
            if ! state.visited.contains(&neighbour)
                && self.1.ensure_freedom_to_move(from, neighbour, false).is_ok()
                && self.1.ensure_constant_contact(from, neighbour, false).is_ok()
            {
                self.reachable_recurse(neighbour, state);
            }
        }
    }
}

// An implementation of DFS for reachability.

impl Perimeter {}
