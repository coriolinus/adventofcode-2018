mod encode_as_u8;
mod input;

use std::path::Path;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub parser); // synthesized by LALRPOP

// each array of 5 bits corresponds to a single number in the range `0..32`,
// so we can encode the complete ruleset as an array of 32 bools.
pub type Rules = [bool; 32];

pub fn part1(input: &Path) -> Result<(), Error> {
    let input = input::Input::new(input)?;
    let rules = input.rules();
    unimplemented!()
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Failed to parse input")]
    Parse(#[from] lalrpop_util::ParseError<usize, String, String>),
    #[error("No solution found")]
    NoSolution,
}
