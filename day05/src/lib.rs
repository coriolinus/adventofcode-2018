extern crate failure;
extern crate intrusive_collections;

pub mod polymer;

use failure::{format_err, Error};
use polymer::Polymer;

pub fn react(p: &mut Polymer) {
    #[cfg(debug_assertions)]
    println!("reacting: {}", polymer::to_string(p));

    let mut cursor = p.cursor_mut();

    loop {
        cursor.move_next();
        if cursor.is_null() {
            break;
        }

        let mut reacts = false;
        {
            let current = cursor.get().unwrap().value;

            #[cfg(debug_assertions)]
            print!("current, next: {}, ", current);

            if let Some(next) = cursor.peek_next().get() {
                let next = next.value;

                #[cfg(debug_assertions)]
                print!("{} ", next);

                if current != next && current.eq_ignore_ascii_case(&next) {
                    reacts = true;
                }
            } else {
                #[cfg(debug_assertions)]
                print!("None ");
            }
        }

        #[cfg(debug_assertions)]
        println!("({})", reacts);

        if reacts {
            cursor.remove();
            cursor.remove();
            // move back two steps so we're at the character before the two removed ones
            cursor.move_prev();
            // but don't move past the beginning
            if cursor.is_null() {
                cursor.move_next();
            }
            cursor.move_prev();
        }
    }
}

fn trim_react(trim: u8, input: &str) -> Result<String, Error> {
    let trim = trim as u8;
    let input: Vec<_> = input
        .bytes()
        .filter(|b| !b.eq_ignore_ascii_case(&trim))
        .collect();
    let mut p = polymer::new(&String::from_utf8(input)?);
    react(&mut p);
    Ok(polymer::to_string(&p))
}

pub fn min_trim_reaction(input: &str) -> Result<String, Error> {
    let mut current = None;

    for trim in b'a'..=b'z' {
        let tr = trim_react(trim, input)?;
        match current {
            None => current = Some(tr),
            Some(c) => {
                current = Some(if tr.len() < c.len() { tr } else { c });
            }
        }
    }

    current.ok_or_else(|| format_err!("no min found"))
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! case {
        ($name:ident($example:expr, $expect:expr)) => {
            #[test]
            fn $name() {
                let mut p = polymer::new($example);
                react(&mut p);
                let result = polymer::to_string(&p);
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
