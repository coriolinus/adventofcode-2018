extern crate day05;
extern crate failure;
extern crate util;

use day05::{polymer, react};
use failure::Error;
use std::fs::read_to_string;
use util::get_input_path;

fn main() -> Result<(), Error> {
    let input = read_to_string(get_input_path())?;
    let mut p = polymer::new(input.trim());
    react(&mut p);
    println!(
        "len after reaction:            {}",
        polymer::to_string(&p).len()
    );
    println!(
        "min len after trim & reaction: {}",
        day05::min_trim_reaction(input.trim())?.len()
    );

    Ok(())
}
