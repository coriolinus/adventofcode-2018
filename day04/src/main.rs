extern crate day04;
extern crate failure;
extern crate util;

use day04::Record;
use failure::Error;
use util::get_input_lines_as;

fn main() -> Result<(), Error> {
    let mut records = get_input_lines_as::<Record>()?;
    records.sort_unstable();
    let mma_guard = Record::most_minutes_asleep(&records).expect("must have well-formed input");
    println!("Guard with most minutes asleep: {}", mma_guard);
    let sm = Record::most_sleepy_minute(&records, mma_guard).expect("must have well-formed input");
    println!("Sleepiest minute: {}", sm);
    let product = mma_guard as u32 * sm as u32;
    println!("Product: {}", product);
    Ok(())
}
