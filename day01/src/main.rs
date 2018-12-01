extern crate day01;
extern crate failure;

use failure::Error;
use std::collections::HashSet;

fn main() -> Result<(), Error> {
    let lines: Vec<i32> = day01::get_input_lines_as()?;
    println!("sum: {}", lines.iter().sum::<i32>());

    let mut sum: i32 = 0;
    let mut states = HashSet::new();
    states.insert(0);
    let mut count = 0;
    for (idx, line) in lines.iter().cycle().enumerate() {
        count = idx;
        sum += line;
        if states.contains(&sum) {
            break;
        }
        states.insert(sum);
    }
    println!("first duplicate: {} (idx: {})", sum, count);
    Ok(())
}
