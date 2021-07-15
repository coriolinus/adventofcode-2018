use aoclib::geometry::{
    map::{ContextFrom, Traversable},
    tile::DisplayWidth,
    Direction, Point,
};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt,
    ops::{Deref, Index},
    path::Path,
    str::FromStr,
};

type UnitPositions = BTreeMap<Point, Unit>;
type HitPoints = i16;

const DEFAULT_ATTACK_POWER: HitPoints = 3;
const DEFAULT_HIT_POINTS: HitPoints = 200;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    parse_display::FromStr,
    parse_display::Display,
)]
enum UnitType {
    #[display("G")]
    Goblin,
    #[display("E")]
    Elf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, parse_display::FromStr, parse_display::Display)]
enum Tile {
    #[display(".")]
    Empty,
    #[display("#")]
    Wall,
    #[display("{0}")]
    Occupied(UnitType),
}

impl DisplayWidth for Tile {
    const DISPLAY_WIDTH: usize = 1;
}

impl ContextFrom<Tile> for Traversable {
    type Context = UnitPositions;

    fn ctx_from(t: Tile, position: Point, context: &Self::Context) -> Self {
        match t {
            Tile::Empty => {
                if context.contains_key(&position) {
                    Traversable::Obstructed
                } else {
                    Traversable::Free
                }
            }
            Tile::Wall | Tile::Occupied(_) => Traversable::Obstructed,
        }
    }
}

type InnerMap = aoclib::geometry::Map<Tile>;

#[derive(Clone)]
struct Map(InnerMap);

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        InnerMap::try_from(std::io::Cursor::new(s))
            .map(Map)
            .map_err(Into::into)
    }
}

impl Deref for Map {
    type Target = InnerMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I> Index<I> for Map
where
    InnerMap: Index<I>,
{
    type Output = <InnerMap as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl Map {
    fn load(input: &Path) -> Result<Self, Error> {
        std::fs::read_to_string(input)?.parse()
    }

    /// Extract the units from this map into their own data structure,
    /// leaving only the immovable tiles of the map.
    fn units(&mut self) -> Units {
        let mut units = Vec::new();
        self.0.for_each_point_mut(|tile, position| {
            if let Tile::Occupied(unit_type) = *tile {
                *tile = Tile::Empty;
                units.push(Unit::new(unit_type, position));
            }
        });
        Units { map: self, units }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Unit {
    unit_type: UnitType,
    position: Point,
    hit_points: HitPoints,
    attack_power: HitPoints,
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
    fn new(unit_type: UnitType, position: Point) -> Unit {
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
    fn turn(&self, map: &Map, positions: &UnitPositions) -> (bool, Option<Point>, Option<Point>) {
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

#[derive(Clone)]
struct Units<'a> {
    map: &'a Map,
    units: Vec<Unit>,
}

impl<'a> Units<'a> {
    /// Perform a round of combat, returning `true` when combat ends due to one side's annihilation.
    fn round(&mut self) -> bool {
        let mut positions: UnitPositions = self
            .units
            .iter()
            .copied()
            .map(|unit| (unit.position, unit))
            .collect();

        // keep track of whether or not combat aborted due to insufficient enemies
        let mut combat_abort = false;

        self.units.sort_unstable();
        // we can't do `for unit in &units` because that would cause a double-borrow conflict when
        // we need to update dead units later.
        for unit_idx in 0..self.units.len() {
            let unit = self.units[unit_idx];
            eprint!(
                "Considering {:?} at ({}, {}) with {} hit points... ",
                unit.unit_type, unit.position.x, unit.position.y, unit.hit_points
            );
            // dead units do nothing
            if unit.hit_points <= 0 {
                eprintln!("which is dead.");
                continue;
            }

            let (end_combat, maybe_move, maybe_attack) = unit.turn(self.map, &positions);
            // handle end of combat
            if end_combat {
                eprintln!("no live enemies found; ending combat");
                combat_abort = true;
                break;
            }
            eprintln!();
            // handle movement
            if let Some(move_to) = maybe_move {
                // we need to remove the unit from the posititions list before moving,
                // and re-add it after.
                debug_assert_eq!(
                    (move_to - unit.position).manhattan(),
                    1,
                    "unit can only move one tile"
                );
                eprintln!("  moving to ({}, {})...", move_to.x, move_to.y);
                // take an owned version of the unit for re-adding
                let mut unit = positions
                    .remove(&unit.position)
                    .expect("positions always correspond to units");
                unit.position = move_to;
                positions.insert(unit.position, unit);
            }
            // handle attacks
            if let Some(attack) = maybe_attack {
                eprintln!(
                    "  attacking target at ({}, {}) with power {}",
                    attack.x, attack.y, unit.attack_power
                );
                let maybe_target = positions.remove(&attack);
                if maybe_target.is_none() {
                    dbg!(attack, unit, &self.units);
                }
                let mut target = maybe_target.expect("positions always correspond to units");
                target.hit_points -= unit.attack_power;
                eprintln!("  -> {:?}", target);

                // note that scanning for targets by position is expensive, so we only
                // do it when the target dies. We have to update the units list entirely
                // at the end of the function anyway.
                if target.hit_points <= 0 {
                    eprintln!("  target dies!");
                    for unit_idx in 0..self.units.len() {
                        if self.units[unit_idx].position == target.position {
                            self.units[unit_idx].hit_points = target.hit_points;
                        }
                    }
                } else {
                    // of course, if the target _didn't_ die, we need to re-add them to the
                    // positions map.
                    positions.insert(target.position, target);
                }
            }
        }

        // whether or not combat aborted, we now need to update the units list
        // units have moved, and their hit points have changed
        self.units = positions
            .into_iter()
            .map(|(_position, unit)| unit)
            .filter(|unit| unit.hit_points > 0)
            .collect();

        eprintln!("{} units survive at round's end", self.units.len());

        combat_abort
    }
}

impl<'a> fmt::Display for Units<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = self.map.0.clone();
        for unit in &self.units {
            map[unit.position] = Tile::Occupied(unit.unit_type);
        }
        write!(f, "{}", map)
    }
}

// known wrong: 68
// known wrong: 69
pub fn part1(input: &Path) -> Result<(), Error> {
    let mut map = Map::load(input)?;
    let mut units = map.units();

    let mut round_count = 0;
    eprintln!("--- starting round {} ---", round_count);
    while !units.round() {
        round_count += 1;
        eprintln!("--- starting round {} ---", round_count);
    }
    println!("combat ended in {} rounds", round_count);
    eprintln!("{}", units);
    Ok(())
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("map conversion")]
    MapConversion(#[from] aoclib::geometry::MapConversionErr),
    #[error("No solution found")]
    NoSolution,
}
