use std::collections::{hash_map::Entry as MapEntry, HashMap, HashSet};

use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// A set of height-sensitive hexes useful for performing reachability calculations.
pub struct Field
{
    map: HashMap<Hex, u8>,
    markers: Collection
}

impl FromIterator<Hex> for Field
{
    fn from_iter<T: IntoIterator<Item = Hex>>(iter: T) -> Self
    {
        let map: HashMap<Hex, u8> = iter.into_iter().map(|h| (h, 1)).collect();

        let mut markers = Collection::default();
        for hex in map.keys()
        {
            markers.insert(*hex);
        }

        Field {
            map,
            markers            
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

impl From<Field> for Collection
{
    fn from(value: Field) -> Self 
    {
        value.markers    
    }
}

impl Field
{
    /// Determines whether this hex is inside the field.
    pub fn contains(&self, h: Hex) -> bool
    {
        self.markers.contains(h)
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

    /// Ensures that the two hexes are neighbours, and returns their common neighbours.
    pub fn ensure_common_neighbours_satisfied(&self, from: Hex, to: Hex) -> Option<(Hex, Hex)>
    {
        hex::common_neighbours(from, to)
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

    /// Boolean of the above
    pub fn ensure_constant_contact_satisfied(&self, from: Hex, to: Hex, ghosting: bool) -> bool
    {
        // Get the common neighbours between the two hexes to ensure they are neighbours.
        let Some((cw, ccw)) = self.ensure_common_neighbours_satisfied(from, to)
        else 
        {
            return false;
        };

        // The height we are moving from is correct, because we haven't decremented the height at that stack yet.
        // The height of the destination should be incremented, because we would end up adding one if this movement were correct.
        let ghosting = if ghosting { 1 } else { 0 };
        let height_f = self.height(from).unwrap_or(1) + ghosting;
        let height_t = self.height(to).map(|h| h + 1).unwrap_or(1);

        if height_f.max(height_t) > 1
        {
            // Then the bug is always touching at least one bug, because either the from-stack or to-stack has a bug underneath and sharing an edge.
            true
        }
        else
        {
            // Otherwise, there is nothing underneath our feet, and we need a neighbour to the side.
            self.contains(cw) || self.contains(ccw)
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

    /// Boolean of the above.
    pub fn ensure_freedom_to_move_satisfied(&self, from: Hex, to: Hex, ghosting: bool) -> bool
    {
        // Get the common neighbours between the two hexes to ensure they are neighbours.
        let Some((cw, ccw)) = self.ensure_common_neighbours_satisfied(from, to)
        else 
        {
            return false;
        };

        if self.contains(cw) && self.contains(ccw)
        {
            let height_cw = self.height(cw).unwrap();
            let height_ccw = self.height(ccw).unwrap();

            let ghosting = if ghosting { 1 } else { 0 };
            let height_f = self.height(from).unwrap_or(1) + ghosting;
            let height_t = self.height(to).map(|h| h + 1).unwrap_or(1);

            let height_path = height_f.max(height_t);
            let height_gate = height_cw.min(height_ccw);

            height_gate < height_path
        }
        else
        {
            false
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

    /// Boolean of the above.
    pub fn ensure_perimeter_crawl_satisfied(&self, from: Hex, to: Hex, distance: Option<u8>) -> bool
    {
        self.find_crawls(from, distance).contains(&to)
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
                self.markers.remove(hex);
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
        if let MapEntry::Occupied(mut o) = self.map.entry(hex)
        {
            *o.get_mut() += 1;
        }
        else 
        {    
            self.map.insert(hex, 1u8);
            self.markers.insert(hex);
        }
    }
}

// An implementation Tarjan's algorithm for finding articulation points.

#[derive(Clone)]
struct DescentRecord
{
    visited: Collection,
    pinned:  Collection,
    num:     [u8; hex::SIZE as usize],
    low:     [u8; hex::SIZE as usize],
    count:   u8,
}

impl Default for DescentRecord
{
    fn default() -> Self
    {
        DescentRecord {
            visited: Collection::default(),
            pinned:  Collection::default(),
            num:     [0; hex::SIZE as usize],
            low:     [0; hex::SIZE as usize],
            count:   0,
        }
    }
}

impl Field
{
    /// Returns all of the pinned hexes.
    pub fn find_pins(&self) -> Collection
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
            Collection::new()
        }
    }

    /// Recursively finds all cut vertices in the field.
    fn find_pins_recurse(&self, hex: Hex, parent: Option<Hex>, state: &mut DescentRecord)
    {
        state.visited.insert(hex);
        state.num[hex as usize] = state.count;
        state.low[hex as usize] = state.count;
        state.count += 1;

        let mut children = 0;
        for neighbour in self.neighbours(hex)
        {
            if let Some(par) = parent
            {
                if par == neighbour
                {
                    continue;
                }
            }

            if state.visited.contains(neighbour)
            {
                state.low[hex as usize] = state.low[hex as usize].min(state.num[neighbour as usize]);
            }
            else
            {
                self.find_pins_recurse(neighbour, Some(hex), state);
                children += 1;

                state.low[hex as usize] = state.low[hex as usize].min(state.low[neighbour as usize]);

                if parent.is_some() && state.low[neighbour as usize] >= state.num[hex as usize]
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
pub struct Perimeter(pub Field, pub Field);

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

        for neighbour in self.0.neighbours(from)
        {
            if !state.visited.contains(&neighbour)
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
