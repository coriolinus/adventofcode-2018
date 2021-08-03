mod point;
mod tile;

use aoclib::geometry::{Direction, Point};
use point::parse_points;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use tile::Tile;

pub type Map = aoclib::geometry::Map<Tile>;

pub const SAFETY_THRESHOLD: i32 = 10_000;

fn make_map(points: &[Point]) -> Map {
    let mut max_x = 0;
    let mut max_y = 0;
    for point in points {
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }

    let mut map = Map::new((max_x + 1) as usize, (max_y + 1) as usize);

    for (idx, point) in points.iter().copied().enumerate() {
        map[point] = Tile::Point(idx);
    }

    map
}

fn fill_map(map: &mut Map, points: &[Point]) -> Result<(), Error> {
    match points.len() {
        0 => return Err(Error::NoSolution),
        1 => map.iter_mut().for_each(|(_position, tile)| {
            if !matches!(tile, Tile::Point(_)) {
                *tile = Tile::Region(0)
            }
        }),
        _ => map.iter_mut().for_each(|(tile_point, tile)| {
            if *tile == Tile::Empty {
                let mut distances = Vec::with_capacity(points.len());

                for (idx, coord) in points.iter().copied().enumerate() {
                    distances.push(((tile_point - coord).manhattan(), idx));
                }

                distances.sort_unstable();

                let (first_dist, idx) = distances[0];
                let (second_dist, _) = distances[1];

                if first_dist == second_dist {
                    // the nearest two coordinates are equidistant
                    *tile = Tile::Equidistant;
                } else {
                    // the nearest coordinate is unique
                    *tile = Tile::Region(idx);
                }
            }
        }),
    }
    debug_assert!(map.iter().all(|(_point, &tile)| matches!(
        tile,
        Tile::Point(_) | Tile::Region(_) | Tile::Equidistant
    )));
    Ok(())
}

fn largest_non_infinite_region(map: &Map) -> Result<usize, Error> {
    let infinite_regions: HashSet<_> = Direction::iter()
        .flat_map(|direction| map.edge(direction))
        .filter_map(|point| match map[point] {
            Tile::Point(idx) | Tile::Region(idx) => Some(idx),
            _ => None,
        })
        .collect();

    let mut region_areas: HashMap<usize, usize> = HashMap::new();
    for tile in map.iter().map(|(_point, tile)| tile).copied() {
        if let Tile::Point(idx) | Tile::Region(idx) = tile {
            if !infinite_regions.contains(&idx) {
                *region_areas.entry(idx).or_default() += 1;
            }
        }
    }

    let mut region_areas: Vec<_> = region_areas.into_iter().collect();
    region_areas.sort_unstable_by_key(|(_idx, area)| std::cmp::Reverse(*area));

    match region_areas.first() {
        Some((_idx, area)) => Ok(*area),
        None => Err(Error::NoSolution),
    }
}

fn size_of_safe_region(map: &Map, points: &[Point]) -> usize {
    map.iter()
        .filter(|&(point, _tile)| {
            points
                .iter()
                .map(|&coord| (coord - point).manhattan())
                .sum::<i32>()
                < SAFETY_THRESHOLD
        })
        .count()
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let points = parse_points(input)?;
    let mut map = make_map(&points);
    fill_map(&mut map, &points)?;
    let area = largest_non_infinite_region(&map)?;

    println!("area of largest non-infinite region: {}", area);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let points = parse_points(input)?;
    let mut map = make_map(&points);
    fill_map(&mut map, &points)?;
    let ssr = size_of_safe_region(&map, &points);

    println!("size of safe region: {}", ssr);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("No solution found")]
    NoSolution,
}
