use aoclib::parse;
use counter::Counter;
use itertools::Itertools;
use std::{path::Path, str::FromStr};

#[derive(Default, Debug)]
struct BoxId {
    word: String,
    freqs: Counter<char>,
}

impl From<String> for BoxId {
    fn from(word: String) -> Self {
        Self {
            freqs: word.chars().collect(),
            word,
        }
    }
}

impl FromStr for BoxId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s.to_string()))
    }
}

impl BoxId {
    fn has_n(&self, n: usize) -> bool {
        self.freqs.values().any(|&v| v == n)
    }
}

pub fn hamming(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).filter(|(a, b)| a != b).count()
}

// This variant of the function iterates over each string twice, but only
// allocates when there's a known match. That turns out to be more performant
// than an implementation which iterates only once but allocates as it goes.
pub fn find_almost_match<S>(strings: &[S]) -> Option<String>
where
    S: AsRef<str>,
{
    strings
        .iter()
        .map(|s| s.as_ref())
        .tuple_combinations()
        .find(|(a, b)| hamming(a, b) == 1)
        .map(|(a, b)| {
            a.chars()
                .zip(b.chars())
                .filter(|(a, b)| a == b)
                .map(|(a, _)| a)
                .collect()
        })
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let ids: Vec<BoxId> = parse(input)?.collect();
    let checksum =
        ids.iter().filter(|id| id.has_n(2)).count() * ids.iter().filter(|id| id.has_n(3)).count();
    println!("checksum: {}", checksum);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let ids: Vec<String> = parse(input)?.collect();
    let almost_match = find_almost_match(&ids).ok_or(Error::NoSolution)?;
    println!("almost match: {}", almost_match);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
