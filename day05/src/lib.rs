

use failure::{format_err, Error};

pub type Polymer = String;

pub fn build_react(s: &str) -> Polymer {
    let mut v = Vec::with_capacity(s.len());
    for &b in s.as_bytes() {
        if v.is_empty() {
            v.push(b);
        } else {
            let terminal = v[v.len() - 1];
            if terminal != b && terminal.eq_ignore_ascii_case(&b) {
                v.pop();
            } else {
                v.push(b);
            }
        }
    }

    Polymer::from_utf8(v).expect("reaction should not destroy ascii-ness")
}

fn trim_react(trim: u8, input: &str) -> Result<String, Error> {
    let trim = trim as u8;
    let input = String::from_utf8(
        input
            .bytes()
            .filter(|b| !b.eq_ignore_ascii_case(&trim))
            .collect(),
    )?;
    Ok(build_react(&input))
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
                let result = build_react($example);
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
