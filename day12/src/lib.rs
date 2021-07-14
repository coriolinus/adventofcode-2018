mod encode_as_u8;
mod input;

use bitvec::vec::BitVec;
use std::{
    ops::{Deref, Index, IndexMut},
    path::Path,
};

// each array of 5 bits corresponds to a single number in the range `0..32`,
// so we can encode the complete ruleset as an array of 32 bools.
pub type Rules = [bool; 32];

struct State {
    pots: BitVec,
    zero_offset: isize,
}

impl Deref for State {
    type Target = BitVec;

    fn deref(&self) -> &Self::Target {
        &self.pots
    }
}

impl Index<isize> for State {
    type Output = bool;

    fn index(&self, index: isize) -> &Self::Output {
        assert!(self.zero_offset + index >= 0, "index out of bounds");
        self.pots.index((self.zero_offset + index) as usize)
    }
}

impl State {
    /// Set a bit of this state.
    fn set(&mut self, index: isize, value: bool) {
        todo!()
    }

    /// Enumerate over all windows of length 5 which are centered
    /// on pots in this state. Treat them as bits of a value, and
    /// return the value.
    ///
    /// The index returned is the offset of the center item in the window.
    ///
    /// Note that this produces 4 more items than `self.len()`: there
    /// are two items overhanging on each side.
    fn windows_enumerated(&self) -> impl Iterator<Item = (isize, u8)> {
        todo!();
        std::iter::empty()
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let input = input::Input::load_file(input)?;
    let rules = input.rules;
    unimplemented!()
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Parse(#[from] pest::error::Error<input::Rule>),
    #[error("No solution found")]
    NoSolution,
}
