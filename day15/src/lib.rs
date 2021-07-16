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

// run a complete combat, returning the winner and the outcome
fn run_combat(units: &mut Units) -> (UnitType, u32) {
    let mut round_count = 0;
    while !units.round() {
        round_count += 1;
    }

    debug_assert!(
        units
            .units
            .windows(2)
            .all(|window| window[0].unit_type == window[1].unit_type),
        "only one unit type must be present at battle end"
    );
    debug_assert!(
        units.units.iter().all(|unit| unit.hit_points > 0),
        "all remaining units must be live",
    );
    debug_assert_ne!(units.units.len(), 0, "complete annihilation is impossible");

    (units.units[0].unit_type, units.outcome(round_count))
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let mut map = Map::load(input)?;
    let mut units = map.units();

    let (_, outcome) = run_combat(&mut units);

    println!("battle outcome: {}", outcome);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let mut map = Map::load(input)?;
    let units = map.units();

    let final_outcome;

    // this has to be a `loop` instead of `for boost in 1..` in order to convince rustc
    // that `final_outcome` is always initialized after termination
    let mut boost = 0;
    loop {
        boost += 1;

        let mut units = units.clone();
        let initial_elf_count = units
            .units
            .iter()
            .filter(|unit| unit.unit_type == UnitType::Elf)
            .count();
        units.set_elf_attack_power(DEFAULT_ATTACK_POWER + boost);
        let (winner, outcome) = run_combat(&mut units);
        if winner == UnitType::Goblin {
            continue;
        }

        // also check that no elves died
        let final_elf_count = units
            .units
            .iter()
            .filter(|unit| unit.unit_type == UnitType::Elf)
            .count();
        if final_elf_count == initial_elf_count {
            final_outcome = outcome;
            break;
        }
    }

    println!("final outcome with min elf boost: {}", final_outcome);
    Ok(())
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
