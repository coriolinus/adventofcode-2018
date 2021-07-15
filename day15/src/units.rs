use crate::{Map, Tile, Unit, UnitPositions};
use std::fmt;

#[derive(Clone)]
pub(crate) struct Units<'a> {
    pub map: &'a Map,
    pub units: Vec<Unit>,
}

impl<'a> Units<'a> {
    /// Perform a round of combat, returning `true` when combat ends due to one side's annihilation.
    pub fn round(&mut self) -> bool {
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
