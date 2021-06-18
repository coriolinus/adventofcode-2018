use aoclib::{geometry::Point, parse};
use std::{borrow::Borrow, path::Path};

type Map = aoclib::geometry::Map<u32>;
const EDGE: usize = 1000;

#[derive(Debug, Clone, parse_display::Display, parse_display::FromStr)]
#[display("#{id} @ {x},{y}: {width}x{height}")]
struct Claim {
    id: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Claim {
    fn iter_points(&self) -> impl '_ + Iterator<Item = Point> {
        (self.y..self.y + self.height).flat_map(move |y| {
            (self.x..self.x + self.width).map(move |x| Point::new(x as i32, y as i32))
        })
    }
}

fn make_map<I, B>(claims: I) -> Map
where
    I: IntoIterator<Item = B>,
    B: Borrow<Claim>,
{
    let mut map = Map::new(EDGE, EDGE);
    for claim in claims {
        for point in claim.borrow().iter_points() {
            map[point] += 1;
        }
    }
    map
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let map = make_map(parse::<Claim>(input)?);
    let n_overlaps = map.iter().filter(|&&used| used > 1).count();
    println!("num overlaps: {}", n_overlaps);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let claims: Vec<Claim> = parse(input)?.collect();
    let map = make_map(&claims);
    let non_overlapping = claims
        .iter()
        .find(|claim| claim.iter_points().all(|point| map[point] == 1))
        .ok_or(Error::NoSolution)?;
    println!("non overlapping claim: {}", non_overlapping.id);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}
