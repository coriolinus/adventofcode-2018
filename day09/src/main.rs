use day09::State;

const INPUT_PLAYERS: usize = 410;
const INPUT_LAST_MARBLE: u32 = 72059;

fn main() {
    let mut s = State::new(INPUT_PLAYERS, INPUT_LAST_MARBLE);
    s.run();
    println!("high score:        {}", s.winner().expect("somebody should win").1);
    let mut s = State::new(INPUT_PLAYERS, INPUT_LAST_MARBLE * 100);
    s.run();
    println!("high score (x100): {}", s.winner().expect("somebody should win").1);
}
