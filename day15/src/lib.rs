mod map;
mod tile;
mod unit;
mod unit_type;
mod units;

use aoclib::geometry::Point;
use std::{collections::BTreeMap, path::Path};
pub(crate) use {map::Map, tile::Tile, unit::Unit, unit_type::UnitType, units::Units};

pub(crate) type UnitPositions = BTreeMap<Point, Unit>;
pub(crate) type HitPoints = i16;

const DEFAULT_ATTACK_POWER: HitPoints = 3;
const DEFAULT_HIT_POINTS: HitPoints = 200;

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
