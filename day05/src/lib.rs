use aoclib::parse;
use bitvec::{bitvec, order::LocalBits};
use std::{path::Path, string::FromUtf8Error};

fn reacts(a: u8, b: u8) -> bool {
    debug_assert!(a.is_ascii_alphabetic());
    debug_assert!(b.is_ascii_alphabetic());
    a != b && a.eq_ignore_ascii_case(&b)
}

/// Perform the entire reaction in a single pass, using two pointers into the
/// input data.
fn react_to_completion(data: &[u8]) -> Vec<u8> {
    if data.len() < 2 {
        return data.into();
    }

    // vector of items which have been reacted / excluded
    let mut exclusions = bitvec![LocalBits, u32; 0; data.len()];

    // two pointers into the data
    let mut lead = 1;
    let mut trail = 0;

    // this is worst case of O(n**2) in the event that all elements are excluded.
    // the simplest way to accomplish that is to ensure that all adjacent elements pair each other.
    // however, I'm pretty confident that the actual length of runs of reactions in the input will
    // be short enough that we don't mind.
    while lead < data.len() {
        // neither lead nor trail must currently be excluded
        debug_assert!(!exclusions[lead]);
        debug_assert!(!exclusions[trail]);

        if reacts(data[lead], data[trail]) {
            // exclude both of these elements; they reacted away
            exclusions.set(lead, true);
            exclusions.set(trail, true);

            // advance the lead
            lead += 1;

            // trail backs up to the most recent non-excluded char
            while trail > 0 && exclusions[trail] {
                trail -= 1;
            }
            // if we run out of chars, we might need to reset entirely
            if trail == 0 && exclusions[trail] {
                trail = lead;
                lead += 1;
            }
        } else {
            trail = lead;
            lead += 1;
        }
    }

    // the output is the input minus all exclusions
    data.iter()
        .copied()
        .zip(exclusions.into_iter())
        .filter_map(|(byte, excluded)| (!excluded).then(move || byte))
        .collect()
}

fn react_str(polymer: String) -> Result<String, Error> {
    String::from_utf8(react_to_completion(polymer.as_bytes())).map_err(Into::into)
}

pub fn part1(input: &Path) -> Result<(), Error> {
    for (idx, data) in parse::<String>(input)?.enumerate() {
        let reacted = react_str(data)?;
        println!("{}: fully reacted len: {}", idx, reacted.len());
    }
    Ok(())
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("No solution found")]
    NoSolution,
    #[error("re-building string from bytes")]
    FromBytes(#[from] FromUtf8Error),
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! case {
        ($name:ident($example:expr, $expect:expr)) => {
            #[test]
            fn $name() {
                let result = react_str($example.into()).unwrap();
                assert_eq!($expect, result);
            }
        };
    }

    case!(two_match("aA", ""));
    case!(two_nomatch("bb", "bb"));
    case!(abba("abBA", ""));
    case!(abab("abAB", "abAB"));
    case!(aabaab("aabAAB", "aabAAB"));
    case!(example("dabAcCaCBAcCcaDA", "dabCBAcaDA"));
    case!(head("YyLlXxYK", "YK"));
}
