use aoclib::parse;

use std::{collections::HashSet, path::Path};

pub type Frequency = i32;

pub fn part1(input: &Path) -> Result<(), Error> {
    let frequency_sum = parse::<Frequency>(input)?.sum::<Frequency>();
    println!("frequency sum: {}", frequency_sum);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let mut states = HashSet::new();
    states.insert(0);
    let mut accumulated = 0;
    let mut count = None;

    for (idx, line) in parse::<Frequency>(input)?
        .collect::<Vec<_>>()
        .into_iter()
        .cycle()
        .enumerate()
    {
        accumulated += line;
        if !states.insert(accumulated) {
            count = Some(idx);
            break;
        }
    }

    let count = count.ok_or(Error::NoSolution)?;

    println!("first duplicate: {} (idx: {})", accumulated, count);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
