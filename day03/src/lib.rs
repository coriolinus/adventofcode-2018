use aoclib::{geometry::Point, parse};
use std::path::Path;

type Map = aoclib::geometry::Map<Vec<u32>>;

#[derive(Debug, Clone, Copy, parse_display::Display, parse_display::FromStr)]
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

pub fn part1(input: &Path) -> Result<(), Error> {
    let mut map = Map::new(1024, 1024);
    for claim in parse::<Claim>(input)? {
        for point in claim.iter_points() {
            map[point].push(claim.id);
        }
    }
    let n_overlaps = map.iter().filter(|used| used.len() > 1).count();
    println!("num overlaps: {}", n_overlaps);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let mut map = Map::new(1024, 1024);
    let claims: Vec<Claim> = parse(input)?.collect();
    for claim in claims.iter() {
        for point in claim.iter_points() {
            map[point].push(claim.id);
        }
    }
    let non_overlapping = claims
        .iter()
        .find(|claim| claim.iter_points().all(|point| map[point].len() == 1))
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
