mod encode_as_u8;
mod input;

use bitvec::prelude::*;
use encode_as_u8::EncodeAsU8;
use std::{
    ops::{Deref, Index},
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
    fn from_initial(initial: BitVec) -> Self {
        Self {
            pots: initial,
            zero_offset: 0,
        }
    }

    /// Set a bit of this state.
    fn set(&mut self, index: isize, value: bool) {
        assert!(self.zero_offset + index >= 0, "index out of bounds");
        self.pots.set((self.zero_offset + index) as usize, value);
    }

    /// Iterate over all pots and relevant indices in the state.
    fn iter_enumerated(&self) -> impl '_ + Iterator<Item = (isize, bool)> {
        std::iter::successors(Some(-self.zero_offset), |&offset| Some(offset + 1))
            .zip(self.pots.iter().by_val())
    }

    /// Enumerate over all windows of length 5 which are centered
    /// on pots in this state. Treat them as bits of a value, and
    /// return the value.
    ///
    /// The index returned is the offset of the center item in the window.
    ///
    /// Note that this produces two more items than `self.len()`.
    /// Normally, windows produces `self.len() - windows - 1` items, and `windows`
    /// here is five. However, this iteration includes windows overhanging
    /// each side by up to 3 values.
    ///
    /// # Panics
    ///
    /// Panics when `self.len() < 5`.
    fn windows_enumerated(&self) -> impl '_ + Iterator<Item = (isize, u8)> {
        assert!(
            self.len() >= 5,
            "can only enumerate windows when at least 5 items present"
        );

        let self_len = self.len();
        let zero_offset = self.zero_offset;

        let left_overhang_3 = [self.pots[0], self.pots[1]].as_u8();
        let left_overhang_2 = [self.pots[0], self.pots[1], self.pots[2]].as_u8();
        let left_overhang_1 = [self.pots[0], self.pots[1], self.pots[2], self.pots[3]].as_u8();
        let left_overhangs =
            std::array::IntoIter::new([left_overhang_3, left_overhang_2, left_overhang_1])
                .enumerate()
                .map(move |(idx, val)| (idx as isize - zero_offset - 1, val));

        let right_overhang_1 = [
            self.pots[self.len() - 4],
            self.pots[self.len() - 3],
            self.pots[self.len() - 2],
            self.pots[self.len() - 1],
            false,
        ]
        .as_u8();
        let right_overhang_2 = [
            self.pots[self.len() - 3],
            self.pots[self.len() - 2],
            self.pots[self.len() - 1],
            false,
            false,
        ]
        .as_u8();
        let right_overhang_3 = [
            self.pots[self.len() - 2],
            self.pots[self.len() - 1],
            false,
            false,
            false,
        ]
        .as_u8();
        let right_overhangs =
            std::array::IntoIter::new([right_overhang_1, right_overhang_2, right_overhang_3])
                .enumerate()
                .map(move |(idx, val)| ((idx + self_len) as isize - zero_offset - 2, val));

        let iteration = self
            .windows(5)
            .enumerate()
            .map(move |(idx, val)| (idx as isize - zero_offset + 2, val.as_u8()));

        left_overhangs.chain(iteration).chain(right_overhangs)
    }

    fn successor(&self, rules: &Rules) -> State {
        let mut succ = State {
            pots: bitvec![0; self.pots.len() + 2],
            zero_offset: self.zero_offset + 1,
        };

        for (idx, val) in self.windows_enumerated() {
            succ.set(idx, rules[val as usize]);
        }

        succ
    }

    fn pot_sum(&self) -> isize {
        self.iter_enumerated()
            .filter_map(|(idx, has_plant)| has_plant.then(move || idx))
            .sum()
    }

    fn into_iter(self, rules: &Rules) -> impl '_ + Iterator<Item = State> {
        std::iter::successors(Some(self), move |state| Some(state.successor(rules)))
    }
}

/// Keep calculating successors until the system settles down into a steady state, as indicated
/// by the difference remaining constant twice in a row.
///
/// Returns `(generation, state, diff of sums)`.
fn advance_until_steady_state(state: State, rules: &Rules) -> (usize, State, isize) {
    let mut old_sum = 0;
    let mut older_sum = 0;

    for (generation, state) in state.into_iter(rules).enumerate() {
        let sum = state.pot_sum();
        let older_diff = old_sum - older_sum;
        let diff = sum - old_sum;
        if diff == older_diff {
            return (generation, state, diff);
        }

        older_sum = old_sum;
        old_sum = sum;
    }

    unreachable!("state is known to stabilize within 1000 iterations")
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let input::Input { rules, initial } = input::Input::load_file(input)?;
    let state = State::from_initial(initial);
    let state = state.into_iter(&rules).skip(20).next().unwrap();
    let pot_sum: isize = state.pot_sum();
    println!("pot sum after 20 generations: {}", pot_sum);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let input::Input { rules, initial } = input::Input::load_file(input)?;
    let state = State::from_initial(initial);
    let (generation, state, diff) = advance_until_steady_state(state, &rules);

    const TARGET_GENERATION: usize = 50_000_000_000;

    let total = state.pot_sum() as usize + (diff as usize * (TARGET_GENERATION - generation));
    println!("pot sum after {} generations: {}", TARGET_GENERATION, total);

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_enumerated_indices() {
        for n_pots in 5..=10 {
            let pots = bitvec![0; n_pots];

            for offset in 0..pots.len() {
                let state = State::from_initial(pots.clone());

                let (indices, values): (Vec<_>, Vec<_>) = state.windows_enumerated().unzip();

                assert!(values.into_iter().all(|value| value == 0));

                assert_eq!(indices.len(), pots.len() + 2);
                assert_eq!(indices[0], -1 - state.zero_offset);
                assert_eq!(
                    *indices.last().unwrap(),
                    pots.len() as isize - state.zero_offset,
                );

                assert!(indices.windows(2).all(|window| window[1] == window[0] + 1));
            }
        }
    }

    #[test]
    fn test_windows_enumerated_values() {
        use std::array::IntoIter;

        for n_pots in 5..=10 {
            let pots: BitVec = IntoIter::new([true, false]).cycle().take(n_pots).collect();
            let expect: Vec<_> = IntoIter::new([0b0010, 0b00101, 0b01010])
                .chain(IntoIter::new([0b10101, 0b01010]).cycle().take(n_pots - 4))
                .chain(IntoIter::new([0b01010, 0b10100, 0b01000]).map(|v| {
                    if n_pots % 2 == 0 {
                        (v << 1) & 0b11111
                    } else {
                        v
                    }
                }))
                .collect();

            for offset in 0..pots.len() {
                let state = State::from_initial(pots.clone());

                let (_, values): (Vec<_>, Vec<_>) = state.windows_enumerated().unzip();
                dbg!(n_pots, offset);
                eprintln!("values = [");
                for value in &values {
                    eprintln!("  {:#07b},", value);
                }
                eprintln!("]");
                assert_eq!(values, expect);
            }
        }
    }
}
