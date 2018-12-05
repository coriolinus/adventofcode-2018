extern crate day05;
extern crate failure;
extern crate util;

use day05::{build_react, min_trim_reaction};
use failure::Error;
use std::fs::read_to_string;
use util::get_input_path;

fn main() -> Result<(), Error> {
    let input = read_to_string(get_input_path())?;
    let p = build_react(input.trim());
    println!("len after reaction:            {}", p.len());
    println!(
        "min len after trim & reaction: {}",
        min_trim_reaction(&p)?.len()
    );

    Ok(())
}
