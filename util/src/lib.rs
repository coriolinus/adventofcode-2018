extern crate failure;

use failure::{Fail, Error};
use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

pub fn get_input_path() -> String {
    args()
        .skip(1)
        .next()
        .unwrap_or(String::from("input.txt"))
}

pub fn get_input() -> Result<BufReader<File>, Error> {
    Ok(BufReader::new(File::open(get_input_path())?))
}

pub fn get_input_lines() -> Result<Vec<String>, Error> {
    let mut out = Vec::new();
    for line in get_input()?.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        out.push(line);
    }

    Ok(out)
}

pub fn get_input_lines_as<T>() -> Result<Vec<T>, Error>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + Send + Sync + Fail,
{
    let mut out = Vec::new();
    for line in get_input()?.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        match line.parse::<T>() {
            Ok(v) => out.push(v),
            Err(e) => {
                eprintln!("unparseable: {}", line);
                return Err(e.into());
            }
        }
    }

    Ok(out)
}
