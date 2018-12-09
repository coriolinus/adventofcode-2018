pub mod circle;
pub mod cursor;

use crate::circle::Circle;
use crate::cursor::Cursor;

#[derive(Debug)]
pub struct State {
    last_marble: u32,
    next_marble: u32,
    next_player: usize,
    scores: Vec<u32>,
    cursor: Cursor<u32>,
}

impl State {
    pub fn new(players: usize, last_marble: u32) -> State {
        // preload the first two steps, which are confusing anyway.
        let mut circle = Circle::with_capacity(last_marble as usize);
        circle.push_back(0);
        circle.push_back(1);

        let mut cursor = Cursor::new(circle);
        cursor.step_right();

        State {
            last_marble,
            next_marble: 2,
            next_player: 2,
            scores: vec![0; players],
            cursor: cursor,
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
            self.cursor.seek(-7);
            self.scores[player] += self.cursor.remove();
        } else {
            self.cursor.seek(2);
            self.cursor.insert(marble);
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
