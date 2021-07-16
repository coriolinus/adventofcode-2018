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
            // dead units do nothing
            if unit.hit_points <= 0 {
                continue;
            }

            let (end_combat, maybe_move, maybe_attack) = unit.turn(self.map, &positions);
            // handle end of combat
            if end_combat {
                combat_abort = true;
                break;
            }
            // handle movement
            if let Some(move_to) = maybe_move {
                // we need to remove the unit from the posititions list before moving,
                // and re-add it after.
                debug_assert_eq!(
                    (move_to - unit.position).manhattan(),
                    1,
                    "unit can only move one tile"
                );
                // take an owned version of the unit for re-adding
                let mut unit = positions
                    .remove(&unit.position)
                    .expect("positions always correspond to units");
                unit.position = move_to;
                positions.insert(unit.position, unit);
            }
            // handle attacks
            if let Some(attack) = maybe_attack {
                let maybe_target = positions.remove(&attack);
                let mut target = maybe_target.expect("positions always correspond to units");
                target.hit_points -= unit.attack_power;

                // note that scanning for targets by position is expensive, so we only
                // do it when the target dies. We have to update the units list entirely
                // at the end of the function anyway.
                if target.hit_points <= 0 {
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

        combat_abort
    }

    pub fn outcome(&self, full_rounds: usize) -> u32 {
        assert!(
            self.units
                .windows(2)
                .all(|window| window[0].unit_type == window[1].unit_type),
            "outcome only reliable when one side is annihilated"
        );
        debug_assert!(
            self.units.iter().all(|unit| unit.hit_points > 0),
            "all remaining units must be live"
        );
        let hit_point_sum: u32 = self.units.iter().map(|unit| unit.hit_points as u32).sum();
        full_rounds as u32 * hit_point_sum
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
