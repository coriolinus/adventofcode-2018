use aoclib::parse;
use std::{collections::VecDeque, path::Path};

#[derive(Debug, parse_display::FromStr, parse_display::Display, Clone, Copy)]
#[display("{players} players; last marble is worth {last_marble} points")]
struct Rules {
    players: usize,
    last_marble: u32,
}

#[derive(Debug)]
pub struct State {
    last_marble: u32,
    next_marble: u32,
    next_player: usize,
    scores: Vec<u32>,
    circle: VecDeque<u32>,
}

impl From<Rules> for State {
    fn from(
        Rules {
            players,
            last_marble,
        }: Rules,
    ) -> Self {
        State::new(players, last_marble)
    }
}

impl State {
    pub fn new(players: usize, last_marble: u32) -> State {
        // preload the first two steps, which are confusing anyway.
        let mut circle = VecDeque::with_capacity(last_marble as usize);
        circle.push_back(0);
        circle.push_back(1);

        State {
            last_marble,
            next_marble: 2,
            next_player: 2,
            scores: vec![0; players],
            circle,
        }
    }

    fn step(&mut self) {
        if self.next_marble > self.last_marble {
            return;
        }

        let marble = self.next_marble;
        self.next_marble += 1;
        let player = self.next_player;
        self.next_player = (self.next_player + 1) % self.scores.len();

        if marble % 23 == 0 {
            self.scores[player] += marble;
            self.circle.rotate_left(7);
            self.scores[player] += self.circle.pop_back().unwrap();
        } else {
            self.circle.rotate_right(2);
            self.circle.push_back(marble);
        }
    }

    pub fn run(&mut self) {
        while self.next_marble <= self.last_marble {
            self.step();
        }
    }

    pub fn winner(&self) -> Option<(usize, u32)> {
        if self.next_marble <= self.last_marble {
            return None;
        }

        self.scores
            .iter()
            .enumerate()
            .map(|(e, &s)| (s, e))
            .max()
            .map(|(s, e)| (e, s))
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    for rules in parse::<Rules>(input)? {
        let mut state: State = rules.into();
        state.run();
        let (_player, winning_score) = state.winner().ok_or(Error::NoSolution)?;

        println!("{} => winning score: {}", rules, winning_score);
    }
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    for mut rules in parse::<Rules>(input)? {
        rules.last_marble *= 100;
        let mut state: State = rules.into();
        state.run();
        let (_player, winning_score) = state.winner().ok_or(Error::NoSolution)?;

        println!("{} => winning score: {}", rules, winning_score);
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("No solution found")]
    NoSolution,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let mut s = State::new(9, 25);
        s.run();
        assert_eq!((5, 32), s.winner().unwrap());
    }

    #[test]
    fn example_1() {
        let mut s = State::new(10, 1618);
        s.run();
        assert_eq!(8317, s.winner().unwrap().1);
    }

    #[test]
    fn example_2() {
        let mut s = State::new(13, 7999);
        s.run();
        assert_eq!(146373, s.winner().unwrap().1);
    }

    #[test]
    fn example_3() {
        let mut s = State::new(17, 1104);
        s.run();
        assert_eq!(2764, s.winner().unwrap().1);
    }

    #[test]
    fn example_4() {
        let mut s = State::new(21, 6111);
        s.run();
        assert_eq!(54718, s.winner().unwrap().1);
    }

    #[test]
    fn example_5() {
        let mut s = State::new(30, 5807);
        s.run();
        assert_eq!(37305, s.winner().unwrap().1);
    }
}
