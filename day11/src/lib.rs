use aoclib::{
    geometry::{Direction, Point},
    parse,
};
use std::{
    iter,
    num::ParseIntError,
    ops::{Deref, Index},
    path::Path,
    str::FromStr,
};

const EDGE_SIZE: usize = 300;

type Map = aoclib::geometry::Map<i32>;

fn power_level(serial: i32, cell: Point) -> i32 {
    debug_assert!(cell.x < EDGE_SIZE as i32);
    debug_assert!(cell.y < EDGE_SIZE as i32);

    // map initialization is 0-indexed, but the coordinate system
    // assumed by the problem statement is 1-indexed
    let cell = Point::new(cell.x + 1, cell.y + 1);

    let rack_id = cell.x + 10;
    let mut power = rack_id * cell.y;
    power += serial;
    power *= rack_id;
    power /= 100;
    power %= 10;
    power -= 5;
    power
}

struct FuelGrid {
    serial: i32,
    map: Map,
}

impl FromStr for FuelGrid {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let serial = s.parse()?;
        Ok(FuelGrid::new(serial))
    }
}

impl Deref for FuelGrid {
    type Target = Map;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<I> Index<I> for FuelGrid
where
    Map: Index<I>,
{
    type Output = <Map as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.map.index(index)
    }
}

impl FuelGrid {
    fn new(serial: i32) -> Self {
        FuelGrid {
            serial,
            map: Map::procedural(EDGE_SIZE, EDGE_SIZE, |cell| power_level(serial, cell)),
        }
    }

    fn first_cell(&self, edge_size: usize) -> FuelCell {
        debug_assert_ne!(edge_size, 0, "edge size must not be zero");

        let mut total_power = 0;
        for x in 0..=edge_size {
            for y in 0..=edge_size {
                total_power += self[(x, y)];
            }
        }

        FuelCell {
            grid: self,
            origin: Point::new(0, 0),
            edge_size,
            total_power,
        }
    }

    /// Iterate over all fuel cells.
    fn fuel_cells(&self, edge_size: usize) -> impl Iterator<Item = FuelCell> {
        let pattern_width = EDGE_SIZE - edge_size;

        // iterate over all cells in a single row, from left to right, moving up after
        let row = iter::repeat(Direction::Right)
            .take(pattern_width)
            .chain(iter::once(Direction::Up));
        // iterate over all cells in a single row, from right to left, moving up after
        let row_back = iter::repeat(Direction::Left)
            .take(pattern_width)
            .chain(iter::once(Direction::Up));
        // the search pattern moves the origin back and forth across rows, moving up once at each end
        let search_pattern = row.chain(row_back).cycle();

        let mut next_cell = Some(self.first_cell(edge_size));

        // This construction is a little awkward; in particular, the sequence `.map(...).take_while(...).map(expect)`
        // doesn't seem ideal. However, it's necessary to ensure that iteration terminates.
        //
        // If we were to just do `.flat_map`, then the underlying search pattern would keep iterating foverver
        // while attempting to find the next non-None item, which we know will never happen.
        search_pattern
            .map(move |direction| {
                let out = next_cell;
                if let Some(cell) = next_cell {
                    next_cell = cell.adjacent(direction);
                }
                out
            })
            .take_while(|maybe_cell| maybe_cell.is_some())
            .map(|cell| cell.expect("known to be Some from check above"))
    }
}

#[derive(Clone, Copy)]
struct FuelCell<'a> {
    grid: &'a FuelGrid,
    origin: Point,
    edge_size: usize,
    total_power: i32,
}

impl<'a> FuelCell<'a> {
    /// Compute the fuel cell adjacent to this one in `direction`.
    ///
    /// Some care has been put into ensuring that we only consider `2 * edge_length` points
    /// when performing this calculation, as opposed to recomputing the new power from scratch.
    ///
    /// This ensures that the time required to perform this calculation is `O(edge_size)`, not
    /// `O(edge_size^2)`.
    fn adjacent(&self, direction: Direction) -> Option<Self> {
        let &FuelCell {
            grid,
            origin,
            edge_size,
            mut total_power,
        } = self;

        // new_origin is where the origin will be after this movement.
        let new_origin = origin + direction;
        // opposit_corner is the new point of this cell farthest from the origin of the cell.
        // we have to substract one so that i.e. if `edge_size == 1` then it matches `new_origin`.
        let opposite_corner = new_origin + Point::new(edge_size as i32 - 1, edge_size as i32 - 1);
        if !grid.in_bounds(new_origin) || !grid.in_bounds(opposite_corner) {
            return None;
        }

        // which way do we go when iterating the added and removed points.
        let iteration_direction = match direction {
            Direction::Down | Direction::Up => Direction::Right,
            Direction::Right | Direction::Left => Direction::Up,
        };
        // `offset_vector` is the direction multiplied by the edge size
        let offset_vector = {
            let (dx, dy) = direction.deltas();
            Point::new(dx, dy) * edge_size as i32
        };

        // construct an iterator of points which were removed from the fuel cell,
        // and remove their power
        let removed_start = match direction {
            Direction::Up | Direction::Right => origin,
            Direction::Down | Direction::Left => new_origin - offset_vector,
        };
        let removed_points = iter::successors(Some(removed_start), |&point| {
            Some(point + iteration_direction)
        })
        .take(edge_size);
        for removed_point in removed_points {
            total_power -= grid[removed_point];
        }

        // construct an iterator of points newly aded to this fuel cell,
        // and add their powers
        let added_start = match direction {
            Direction::Up | Direction::Right => origin + offset_vector,
            Direction::Down | Direction::Left => new_origin,
        };
        let added_points = iter::successors(Some(added_start), |&point| {
            Some(point + iteration_direction)
        })
        .take(edge_size);
        for added_point in added_points {
            total_power += grid[added_point];
        }

        Some(FuelCell {
            grid,
            origin: new_origin,
            edge_size,
            total_power,
        })
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    for fuel_grid in parse::<FuelGrid>(input)? {
        let max_power_cell = fuel_grid
            .fuel_cells(3)
            .max_by_key(|cell| cell.total_power)
            .expect("fuel grid is never empty");
        // offset by 1 because AoC expects 1-indexing for this problem
        let coords = max_power_cell.origin + Point::new(1, 1);
        println!(
            "for serial {}: origin of max power cell: {},{}",
            fuel_grid.serial, coords.x, coords.y
        );
    }
    Ok(())
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("parsing input integer")]
    ParseInt(#[from] ParseIntError),
    #[error("No solution found")]
    NoSolution,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_power_level(serial: i32, coords: (usize, usize), expect: i32) {
        assert_ne!(coords.0, 0);
        assert_ne!(coords.1, 0);

        // decrement the x and y coords by 1, because that's how the map will do it
        let coords = Point::new(coords.0 as i32 - 1, coords.1 as i32 - 1);
        assert_eq!(power_level(serial, coords), expect);
    }

    #[test]
    fn example() {
        check_power_level(8, (3, 5), 4);
    }

    #[test]
    fn example_1() {
        check_power_level(57, (122, 79), -5);
    }

    #[test]
    fn example_2() {
        check_power_level(39, (217, 196), 0);
    }

    #[test]
    fn example_3() {
        check_power_level(71, (101, 153), 4);
    }
}
