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
    let mut count = 0;
    let mut sum = 0;
    for (idx, line) in parse::<Frequency>(input)?
        .collect::<Vec<_>>()
        .into_iter()
        .cycle()
        .enumerate()
    {
        sum += line;
        if !states.insert(sum) {
            count = idx;
            break;
        }
    }
    println!("first duplicate: {} (idx: {})", sum, count);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
