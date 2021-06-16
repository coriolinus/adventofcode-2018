


use counter::Counter;
use itertools::Itertools;
use std::hash::Hash;

trait Hashable {
    fn has_n(&self, n: usize) -> bool;
}

impl<T> Hashable for Counter<T>
where
    T: Hash + Eq,
{
    fn has_n(&self, n: usize) -> bool {
        for v in self.values() {
            if v == &n {
                return true;
            }
        }
        false
    }
}

pub fn hash<I>(strings: &[I]) -> usize
where
    I: AsRef<str>,
{
    let mut twos = 0;
    let mut threes = 0;

    for as_s in strings {
        let s = as_s.as_ref();
        let c = Counter::init(s.chars());
        if c.has_n(2) {
            twos += 1;
        }
        if c.has_n(3) {
            threes += 1;
        }
    }

    twos * threes
}

pub fn hamming(a: &str, b: &str) -> usize {
    if a.len() != b.len() {
        return std::cmp::max(a.len(), b.len());
    }
    let mut hd = 0;
    for (a, b) in a.chars().zip(b.chars()) {
        if a != b {
            hd += 1;
        }
    }
    hd
}

pub fn find_almost_match<I>(strings: &[I]) -> Option<String>
where
    I: AsRef<str>,
{
    for (a, b) in strings.iter().tuple_combinations() {
        let (a, b) = (a.as_ref(), b.as_ref());

        if hamming(a, b) == 1 {
            return Some(
                a.chars()
                    .zip(b.chars())
                    .filter_map(|(a, b)| if a == b { Some(a) } else { None })
                    .collect(),
            );
        }
    }
    None
}
