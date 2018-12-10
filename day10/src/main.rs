use day10::{display, find_min_area, Point};
use failure::format_err;
use util::get_input_lines_as;

fn main() -> Result<(), failure::Error> {
    let points = get_input_lines_as::<Point>().map_err(|_| format_err!("parse fail"))?;
    let (smallest, seconds) = find_min_area(&points);
    println!("{}", display(&smallest).unwrap());
    println!("after {} seconds", seconds);
    Ok(())
}
