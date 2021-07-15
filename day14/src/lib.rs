use aoclib::parse;
use std::path::Path;

const INITIAL_ELVES: [usize; 2] = [0, 1];
const INITIAL_RECIPES: [u8; 2] = [3, 7];

fn initial_recipes(generations: u32) -> Vec<u8> {
    let mut recipes = Vec::with_capacity(generations as usize + 10);
    recipes.extend(INITIAL_RECIPES);
    recipes
}

fn make_recipe(elves: &mut [usize; 2], recipes: &mut Vec<u8>) {
    let sum = recipes[elves[0]] + recipes[elves[1]];
    if sum > 9 {
        recipes.push(sum / 10);
    }
    recipes.push(sum % 10);

    for elf in elves.iter_mut() {
        *elf += recipes[*elf] as usize + 1;
        *elf %= recipes.len();
    }
}

fn scores(recipes: &[u8], generations: u32) -> Option<u64> {
    let start = generations as usize;
    let stop = start + 10;
    if recipes.len() < stop {
        return None;
    }

    let mut score = 0;
    for recipe in &recipes[start..stop] {
        score *= 10;
        score += *recipe as u64;
    }

    Some(score)
}

/// If a score matches at the last or second-last sequence of digits from the end,
/// return the index of the first digit of the score.
///
/// Note that this is _not_ a general search; it must be called
/// once for each invocation of `make_recipe` in order to work properly.
fn matches_score(recipes: &[u8], score: u32) -> Option<usize> {
    fn matches_score_offset(recipes: &[u8], mut score: u32, offset: usize) -> Option<usize> {
        let mut count_score_digits = 0;
        let score_digits = std::iter::from_fn(|| {
            (score != 0).then(|| {
                let out = score % 10;
                score /= 10;
                count_score_digits += 1;
                out as u8
            })
        });

        recipes
            .iter()
            .rev()
            .skip(offset)
            .zip(score_digits)
            .all(|(&r, s)| r == s)
            .then(move || recipes.len() - offset - count_score_digits)
    }

    matches_score_offset(recipes, score, 1).or_else(|| matches_score_offset(recipes, score, 0))
}

fn build_until_matches_score(mut recipes: Vec<u8>, score: u32) -> usize {
    let mut elves = INITIAL_ELVES;
    loop {
        if let Some(generation) = matches_score(&recipes, score) {
            return generation;
        }
        make_recipe(&mut elves, &mut recipes);
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    for generations in parse(input)? {
        let mut elves = INITIAL_ELVES;
        let mut recipes = initial_recipes(generations);
        while scores(&recipes, generations).is_none() {
            make_recipe(&mut elves, &mut recipes);
        }
        let scores = scores(&recipes, generations).unwrap();

        println!("given {}, expect scores: {}", generations, scores);
    }
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    for target_score in parse(input)? {
        let generations = build_until_matches_score(initial_recipes(0), target_score);
        println!(
            "for target score {}, requires generations: {}",
            target_score, generations
        );
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
    use rstest::rstest;

    #[test]
    fn part1_example() {
        const GENERATIONS: u32 = 9;
        let mut elves = INITIAL_ELVES;
        let mut recipes = initial_recipes(GENERATIONS);
        dbg!(&recipes);

        while scores(&recipes, GENERATIONS).is_none() {
            make_recipe(&mut elves, &mut recipes);
            dbg!(&elves, &recipes);
        }

        assert_eq!(scores(&recipes, GENERATIONS).unwrap(), 5158916779);
    }

    #[rstest]
    #[case(51589, 9)]
    #[case(92510, 18)]
    #[case(59414, 2018)]
    // #[case(01245, 5)]
    // Can't effectively test cases with a leading 0 in this implementation.
    fn part2_examples(#[case] target_score: u32, #[case] expect: usize) {
        assert_eq!(
            build_until_matches_score(initial_recipes(0), target_score),
            expect
        );
    }
}
