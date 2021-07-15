use crate::{
    HitPoints, Map, Tile, UnitPositions, UnitType, DEFAULT_ATTACK_POWER, DEFAULT_HIT_POINTS,
};
use aoclib::geometry::{Direction, Point};
use std::{cmp::Ordering, collections::BTreeMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Unit {
    pub unit_type: UnitType,
    pub position: Point,
    pub hit_points: HitPoints,
    pub attack_power: HitPoints,
}

impl Ord for Unit {
    // reading order: first down, then over, then incidentals
    fn cmp(&self, other: &Self) -> Ordering {
        self.position
            .y
            .cmp(&other.position.y)
            .reverse()
            .then_with(|| self.position.x.cmp(&other.position.x))
            .then_with(|| self.unit_type.cmp(&other.unit_type))
            .then_with(|| self.hit_points.cmp(&other.hit_points))
            .then_with(|| self.attack_power.cmp(&other.attack_power))
    }
}

impl PartialOrd for Unit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Unit {
    pub fn new(unit_type: UnitType, position: Point) -> Unit {
        Self {
            unit_type,
            position,
            hit_points: DEFAULT_HIT_POINTS,
            attack_power: DEFAULT_ATTACK_POWER,
        }
    }

    /// Take a unit's turn.
    ///
    /// Turns proceed in this sequence.
    ///
    /// 1. Identify targets. If no targets, combat ends. Targets are those whose type differs.
    /// 2. If unit is in range of a target, do not move, but proceed to attack.
    /// 3. If unit is not in range of a target, proceed to move.
    ///    a. Identify squares that are in range of targets and empty.
    ///    b. Determine which of them can be reached in orthogonal steps without moving through
    ///       any unit's current position, or walls.
    ///    c. If no positions can be reached, end turn without moving.
    ///    d. If multiple positions are tied for least steps, choose the first by reading order.
    ///    e. If there is a unique target position reachable in least steps, choose it.
    ///    f. Take a single step along the shortest path to that destination. (If there is
    ///       more than one shortest path, choose the first step with the best reading order.)
    /// 4. Attack.
    ///    a. Determine all targets which are in range (adjacent). If none, end turn.
    ///    b. Select target with fewest hit points. In case of tie, choose the least by reading order.
    ///    c. Reduce target's hit points by attack power.
    ///    d. If target's hit points are 0 or lower, it dies; remove it from play.
    ///
    /// This method holds an immutable reference to its struct. It returns a bool and up to two `Point`s:
    /// `(combat_ends, move, attack_target)`. It is the caller's responsibility to update global state
    /// appropriately with those outputs.
    ///
    /// The map stores geographic features but must not contain any units.
    /// The `UnitPositions` struct stores all units' positions.
    pub fn turn(
        &self,
        map: &Map,
        positions: &UnitPositions,
    ) -> (bool, Option<Point>, Option<Point>) {
        debug_assert!(
            !map.iter().any(|&tile| matches!(tile, Tile::Occupied(_))),
            "map must have only geography"
        );
        debug_assert!(
            !positions
                .iter()
                .any(|(&position, unit)| position != unit.position),
            "positions map must agree with itself"
        );

        // 1. If no targets, combat ends.
        let all_targets: Vec<_> = self.targets(positions).collect();
        if all_targets.is_empty() {
            return (true, None, None);
        }

        // 2. If unit is in range of a target, do not move, but proceed to attack.
        let find_adjacent_targets = |position: Point| {
            map.orthogonal_adjacencies(position)
                .filter(move |position| {
                    positions
                        .get(position)
                        .map(|unit| unit.unit_type != self.unit_type)
                        .unwrap_or_default()
                })
                .collect()
        };
        let mut adjacent_targets: Vec<_> = find_adjacent_targets(self.position);
        let move_to = adjacent_targets
            .is_empty()
            .then(|| self.compute_move(all_targets, map, positions))
            .flatten();
        if let Some(dest) = move_to {
            // we've moved, recompute the targets
            adjacent_targets = find_adjacent_targets(dest);
        }

        // 3. Attack.
        let attack = self.attack(adjacent_targets, positions);

        (false, move_to, attack)
    }

    /// Identify potential targets by their position.
    fn targets<'a>(&'a self, positions: &'a UnitPositions) -> impl 'a + Iterator<Item = Point> {
        positions
            .iter()
            .filter_map(move |(point, unit)| (unit.unit_type != self.unit_type).then(move || point))
            .copied()
    }

    /// Move a step as specified in the rules. Returns the point to which we're moving.
    ///
    /// a. Identify squares that are in range of targets and empty.
    /// b. Determine which of them can be reached in orthogonal steps without moving through
    ///    any unit's current position, or walls.
    /// c. If no positions can be reached, end turn without moving.
    /// d. If multiple positions are tied for least steps, choose the first by reading order.
    /// e. If there is a unique target position reachable in least steps, choose it.
    /// f. Take a single step along the shortest path to that destination. (If there is
    ///    more than one shortest path, choose the first step with the best reading order.)
    fn compute_move(
        &self,
        targets: Vec<Point>,
        map: &Map,
        positions: &UnitPositions,
    ) -> Option<Point> {
        // identify squares that are in range of targets adn empty
        // determine which of them can be reached without obstruction
        let targets = Self::in_range_and_empty(targets.into_iter(), map, positions).filter_map(
            |destination| {
                map.navigate_ctx(positions, self.position, destination)
                    .map(|directions| (directions.len(), destination))
            },
        );
        // determine the destination which can be reached in fewest steps
        let mut steps_to_target = BTreeMap::<_, Vec<_>>::new();
        for (steps_to, target) in targets {
            steps_to_target.entry(steps_to).or_default().push(target);
        }
        let (dist, mut nearest_targets) = steps_to_target.into_iter().next()?;
        // if multiple are tied for least steps, choose by reading order
        nearest_targets.sort_unstable();
        let destination = *nearest_targets.first()?;
        // determine which path to the destination is shortest by reading order
        let first_step = std::array::IntoIter::new([
            Direction::Up,
            Direction::Left,
            Direction::Right,
            Direction::Down,
        ])
        .find_map(|direction| {
            let adjacent_point = self.position + direction;
            if map[adjacent_point] != Tile::Empty || positions.contains_key(&adjacent_point) {
                return None;
            }
            let steps_to = map.navigate_ctx(positions, adjacent_point, destination)?;
            (steps_to.len() == dist - 1).then(move || direction)
        })
        .expect("at least one direction must be the first direction on the path");

        Some(self.position + first_step)
    }

    /// Attack per the instructions.
    ///
    /// a. Determine all targets which are in range (adjacent). If none, end turn.
    /// b. Select target with fewest hit points. In case of tie, choose the least by reading order.
    /// c. ~~Reduce target's hit points by attack power.~~
    /// d. ~~If target's hit points are 0 or lower, it dies; remove it from play.~~
    fn attack(&self, mut targets: Vec<Point>, positions: &UnitPositions) -> Option<Point> {
        // first sort by reading order, then (stably) by hit points, so hit points have higher priority
        targets.sort_unstable();
        targets.sort_by_key(|target| positions[target].hit_points);
        targets.first().copied()
    }

    /// Positions adjacent to targets which are in range and empty.
    fn in_range_and_empty<'a>(
        target_positions: impl 'a + Iterator<Item = Point>,
        map: &'a Map,
        positions: &'a UnitPositions,
    ) -> impl 'a + Iterator<Item = Point> {
        target_positions
            .flat_map(move |point| map.orthogonal_adjacencies(point))
            .filter(move |&point| map[point] == Tile::Empty && !positions.contains_key(&point))
    }
}
