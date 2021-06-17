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
// allocates when there's a known match.
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

/// The elements common to `a` and `b` if there is only one difference
pub fn almost_match((a, b): (&str, &str)) -> Option<String> {
    let mut diffs = 0;
    let common = a
        .chars()
        .zip(b.chars())
        .filter_map(|(a, b)| {
            if a == b {
                Some(a)
            } else {
                diffs += 1;
                None
            }
        })
        .collect();
    (diffs == 1).then(move || common)
}

// This variant of the function iterates over each string only once,
// but builds up the common elements into a new string each time.
pub fn find_almost_match_mode_2<S>(strings: &[S]) -> Option<String>
where
    S: AsRef<str>,
{
    strings
        .iter()
        .map(|s| s.as_ref())
        .tuple_combinations()
        .find_map(almost_match)
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

pub fn part2_mode2(input: &Path) -> Result<(), Error> {
    let ids: Vec<String> = parse(input)?.collect();
    let almost_match = find_almost_match_mode_2(&ids).ok_or(Error::NoSolution)?;
    println!("almost match: {}", almost_match);
    Ok(())
}

// Hyperfine results comparing part2 basic mode to part2 mode2:
//
// Benchmark #1: target/release/day02 --no-part1 --part2
//   Time (mean ± σ):       1.2 ms ±   0.1 ms    [User: 1.2 ms, System: 0.1 ms]
//   Range (min … max):     1.1 ms …   1.8 ms    1814 runs
//
//   Warning: Command took less than 5 ms to complete. Results might be inaccurate.
//   Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet PC without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.
//
// Benchmark #2: target/release/day02 --no-part1 --part2-mode2
//   Time (mean ± σ):       1.6 ms ±   0.1 ms    [User: 1.5 ms, System: 0.1 ms]
//   Range (min … max):     1.4 ms …   2.1 ms    1552 runs
//
//   Warning: Command took less than 5 ms to complete. Results might be inaccurate.
//
// Summary
//   'target/release/day02 --no-part1 --part2' ran
//     1.28 ± 0.09 times faster than 'target/release/day02 --no-part1 --part2-mode2'

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
