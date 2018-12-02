extern crate day01;
extern crate day02;
extern crate failure;

use day01::get_input_lines;
use day02::{find_almost_match, hash};
use failure::Error;

fn main() -> Result<(), Error> {
    let lines = get_input_lines()?;
    println!("hash:   {}", hash(&lines));
    println!("common: {:?}", find_almost_match(&lines));
    Ok(())
}
