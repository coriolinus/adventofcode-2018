pub const GRID_SIZE: usize = 300;
pub type FuelCell = i8;
pub type FuelGrid = [[FuelCell; GRID_SIZE]; GRID_SIZE];

#[inline]
fn initialize_cell(serial: u32, coordinates: (usize, usize)) -> FuelCell {
    let (x, y) = coordinates;
    let rack_id = x as u32 + 10;
    let power = ((y as u32 * rack_id) + serial) * rack_id;
    let pwr_s = power.to_string();
    let power: FuelCell = if pwr_s.len() >= 3 {
        pwr_s
            .chars()
            .rev()
            .skip(2)
            .next()
            .unwrap() // safe: we already checked pwr_s.len()
            .to_string()
            .parse()
            .unwrap() // safe: we know this string is comprised of digits
    } else {
        0
    };

    power - 5
}

pub fn initialize(serial: u32) -> FuelGrid {
    let mut grid = [[0; GRID_SIZE]; GRID_SIZE];

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            grid[y][x] = initialize_cell(serial, (x + 1, y + 1));
        }
    }

    grid
}

// reduce a sums grid by one step
//
// assumes the grids are square
//
// first, identify the current difference in edge length between the grid and the sums grid.
// this is the OFFSET. increase the offset by 1.
fn reduce_sgrid(_grid: &FuelGrid, _sgrid: &mut Vec<Vec<i32>>) {
    unimplemented!()
}

pub fn compute_max(grid: &FuelGrid) -> (usize, usize) {
    let mut totals = [[0; GRID_SIZE]; GRID_SIZE];

    for y in 1..(GRID_SIZE - 1) {
        for x in 1..(GRID_SIZE - 1) {
            totals[y][x] += grid[y - 1][x - 1];
            totals[y][x] += grid[y - 1][x];
            totals[y][x] += grid[y - 1][x + 1];
            totals[y][x] += grid[y][x - 1];
            totals[y][x] += grid[y][x];
            totals[y][x] += grid[y][x + 1];
            totals[y][x] += grid[y + 1][x - 1];
            totals[y][x] += grid[y + 1][x];
            totals[y][x] += grid[y + 1][x + 1];
        }
    }

    totals
        .iter()
        .enumerate()
        .map(|(y, row)| {
            (
                row.iter()
                    .enumerate()
                    .map(|(x, val)| (val, x))
                    .max()
                    .unwrap(),
                y,
            )
        })
        .max()
        .map(|((_, x), y)| (x, y))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        assert_eq!(initialize_cell(8, (3, 5)), 4);
    }

    #[test]
    fn example_1() {
        assert_eq!(initialize_cell(57, (122, 79)), -5);
    }

    #[test]
    fn example_2() {
        assert_eq!(initialize_cell(39, (217, 196)), 0);
    }

    #[test]
    fn example_3() {
        assert_eq!(initialize_cell(71, (101, 153)), 4);
    }

    // #[test]
    // fn max_with() {
    //     for serial in &[123, 234, 345, 456] {
    //         let grid = initialize(*serial);
    //         assert_eq!(compute_max(&grid), compute_max(&grid, 3));
    //     }
    // }

    // #[test]
    // fn abs_max_example() {
    //     let grid = initialize(18);
    //     assert_eq!((90, 269, 16), find_abs_max(&grid));
    // }
}
