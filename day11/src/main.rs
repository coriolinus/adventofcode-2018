const SERIAL: u32 = 5153;
use day11::*;

fn main() {
    let grid = initialize(SERIAL);
    let (x, y) = compute_max(&grid);
    println!("cell with max power (sz. 3): ({}, {})", x, y);
    // println!("abs max: {:?}", find_abs_max(&grid));
}
