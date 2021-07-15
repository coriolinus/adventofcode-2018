use aoclib::geometry::{tile::DisplayWidth, Point};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
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

#[derive(Debug, Clone, Copy, parse_display::FromStr, parse_display::Display)]
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
        todo!()
    }

    /// Identify potential targets by their position.
    fn targets<'a>(&'a self, positions: &'a UnitPositions) -> impl 'a + Iterator<Item = Point> {
        positions
            .iter()
            .filter_map(move |(point, unit)| (unit.unit_type != self.unit_type).then(move || point))
            .copied()
    }
}

#[derive(Clone)]
struct Units<'a> {
    map: &'a Map,
    units: Vec<Unit>,
}

impl<'a> Units<'a> {
    fn round(&mut self) {
        todo!()
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    unimplemented!()
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
