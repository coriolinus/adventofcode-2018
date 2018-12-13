use lalrpop_util::lalrpop_mod;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

lalrpop_mod!(pub input); // synthesized by LALRPOP

pub type Rule = ([bool; 5], bool);
pub type Rules = HashMap<[bool; 5], bool>;

pub struct Input {
    pub initial: Vec<bool>,
    pub rules: Vec<Rule>,
}

impl Input {
    pub fn rules(&self) -> Rules {
        self.rules.iter().cloned().collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    s: Vec<bool>,
    offset: usize,
}

impl<Bools> From<Bools> for State
where
    Bools: AsRef<[bool]>,
{
    fn from(bools: Bools) -> State {
        State {
            s: bools.as_ref().to_vec(),
            ..State::default()
        }
    }
}

impl Index<isize> for State {
    type Output = bool;

    fn index(&self, idx: isize) -> &bool {
        if idx < 0 && (idx.abs() as usize) > self.offset {
            return &false;
        }

        if (idx as usize) + self.offset > self.s.len() {
            return &false;
        }

        let idx = (idx + (self.offset as isize)) as usize;
        &self.s[idx]
    }
}

impl IndexMut<isize> for State {
    fn index_mut<'a>(&'a mut self, mut idx: isize) -> &'a mut bool {
        if idx < 0 && (idx.abs() as usize) > self.offset {
            let new_offset = idx.abs() as usize;
            let growl = new_offset - self.offset;
            let mut news = vec![false; growl + self.s.len()];
            news[new_offset..].copy_from_slice(&self.s[self.offset..]);

            self.s = news;
            self.offset = new_offset;
            idx += growl as isize;
        } else if idx > 0 && (idx as usize) + self.offset > self.s.len() {
            self.s.resize((idx as usize) + self.offset, false);
        }

        let idx = (idx + (self.offset as isize)) as usize;
        &mut self.s[idx]
    }
}

impl State {
    pub fn len(&self) -> isize {
        self.s.len() as isize
    }

    fn array_for(&self, idx: usize) -> [bool; 5] {
        let mut array = [false; 5];

        match idx {
            0 => array[2..5].copy_from_slice(&self.s[0..3]),
            1 => array[1..5].copy_from_slice(&self.s[0..4]),
            idx if idx == self.s.len() - 2 => {
                array[..3].copy_from_slice(&self.s[(self.s.len() - 3)..])
            }
            idx if idx == self.s.len() - 1 => {
                array[..4].copy_from_slice(&self.s[(self.s.len() - 4)..])
            }
            idx if idx >= self.s.len() => panic!("idx {} out of bounds"),
            idx => array[..].copy_from_slice(&self.s[idx - 2..=idx + 2]),
        };

        array
    }

    pub fn step(&self, rules: &Rules) -> State {
        let mut next = self.clone();
        for idx in 0..next.len() {
            next[idx] = *rules.get(&self.array_for(idx as usize)).unwrap();
        }
        next
    }

    pub fn sum_indices(&self, offset: usize) -> i64 {
        self.s
            .iter()
            .enumerate()
            .filter_map(|(i, &v)| {
                if v {
                    Some(i as i64 - offset as i64)
                } else {
                    None
                }
            })
            .sum()
    }
}

pub fn steps(state: &State, rules: &Rules, count: usize) -> State {
    let mut state = state.clone();
    let mut next: State;

    for _ in 0..count {
        next = state.step(rules);
        std::mem::swap(&mut state, &mut next);
    }

    state
}
