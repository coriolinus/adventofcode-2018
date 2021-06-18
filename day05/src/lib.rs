use aoclib::{data_structures::linked_list::LinkedList, parse};
use std::{path::Path, string::FromUtf8Error};

fn reacts(a: u8, b: u8) -> bool {
    debug_assert!(a.is_ascii_alphabetic());
    debug_assert!(b.is_ascii_alphabetic());
    a != b && a.eq_ignore_ascii_case(&b)
}

fn react_to_completion(data: impl IntoIterator<Item = u8>) -> Vec<u8> {
    let mut list: LinkedList<_> = data.into_iter().collect();

    // keep looping through the list as long as the length keeps changing
    let mut len_before = !0;
    while len_before != list.len() {
        len_before = list.len();
        let mut cursor = list.cursor_front();

        // for each cursor position, if there is a match with the successor,
        // clear this position and the next
        loop {
            match (cursor.elem().copied(), cursor.peek_next().copied()) {
                (Some(elem), Some(next)) if reacts(elem, next) => {
                    dbg!(elem as char, next as char);
                    // clear the members of the reaction
                    // taking moves the cursor forward if possible
                    cursor.take();
                    cursor.take();
                }
                (Some(_), Some(_)) => {
                    // these aren't reactive chemicals, advance the cursor
                    dbg!(cursor.advance().map(|byte| *byte as char));
                }
                _ => {
                    // we're at the end of the list
                    break;
                }
            }
        }
    }

    list.iter().copied().collect()
}

fn react_str(polymer: String) -> Result<String, Error> {
    String::from_utf8(react_to_completion(polymer.as_bytes().iter().copied())).map_err(Into::into)
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
