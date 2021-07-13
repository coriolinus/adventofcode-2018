use aoclib::{
    geometry::{tile::Bool, Map, Point},
    parse,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::{num::ParseIntError, path::Path, str::FromStr};

lazy_static! {
    static ref POINT_RE: Regex = Regex::new(r"<\s?(?P<x>-?\d+), \s?(?P<y>-?\d+)>").unwrap();
    static ref LIGHT_RE: Regex =
        Regex::new(r"position=(?P<position><[-\d ,]+>) velocity=(?P<velocity><[-\d ,]+>)").unwrap();
}

fn parse_point(s: &str) -> Result<Point, Error> {
    let captures = POINT_RE.captures(s).ok_or(Error::ParseError)?;
    let x = captures
        .name("x")
        .expect("x always in captures")
        .as_str()
        .parse()?;
    let y = captures
        .name("y")
        .expect("y always in captures")
        .as_str()
        .parse()?;
    Ok(Point::new(x, y))
}

#[derive(Clone, Copy, Debug)]
struct Light {
    position: Point,
    velocity: Point,
}

impl FromStr for Light {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = LIGHT_RE.captures(s).ok_or(Error::ParseError)?;
        let position = captures
            .name("position")
            .expect("position always in captures")
            .as_str();
        let position = parse_point(position)?;
        let velocity = captures
            .name("velocity")
            .expect("velocity always in captures")
            .as_str();
        let velocity = parse_point(velocity)?;

        Ok(Light { position, velocity })
    }
}

/// Compute the `(min, max)` bounds enclosing the given points.
fn bounds(points: &[Light]) -> (Point, Point) {
    let mut min = Point::new(i32::MAX, i32::MAX);
    let mut max = Point::new(i32::MIN, i32::MIN);
    for point in points {
        min.x = min.x.min(point.position.x);
        min.y = min.y.min(point.position.y);
        max.x = max.x.max(point.position.x);
        max.y = max.y.max(point.position.y);
    }

    (min, max)
}

/// Compute the bounding area of the given points.
fn area(points: &[Light]) -> u64 {
    let (min, max) = bounds(points);
    debug_assert!(max.x >= min.x);
    debug_assert!(max.y >= min.y);
    let width = (max.x - min.x) as u64;
    let height = (max.y - min.y) as u64;
    width * height
}

// advance the state of the lights
fn tick(lights: &mut [Light]) {
    for light in lights.iter_mut() {
        light.position += light.velocity;
    }
}

fn find_min_area(mut lights: Vec<Light>) -> (Vec<Light>, usize) {
    let mut count = 0;

    let mut prev_area = area(&lights);
    tick(&mut lights);
    let mut current_area = area(&lights);

    while current_area <= prev_area {
        prev_area = current_area;
        tick(&mut lights);
        current_area = area(&lights);
        count += 1;
    }

    for light in lights.iter_mut() {
        light.position -= light.velocity;
    }

    (lights, count)
}

fn to_map(mut lights: Vec<Light>) -> Map<Bool> {
    // adjust the lights such that the minimum corner is at `(0, 0)`
    let (min, max) = bounds(&lights);
    for light in lights.iter_mut() {
        light.position -= min;
    }

    let max = max - min;

    debug_assert_eq!(
        bounds(&lights),
        (Point::new(0, 0), max),
        "adjustment must minimize positions",
    );
    // convert into a map
    let mut map = Map::new(max.x as usize + 1, max.y as usize + 1);
    for light in lights {
        map[light.position] = Bool::True;
    }

    // map's origin is in bottom left, but AoC coords assume origin is in top left
    map = map.flip_vertical();

    map
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let lights: Vec<Light> = parse(input)?.collect();
    let (min_area_lights, _) = find_min_area(lights);
    let map = to_map(min_area_lights);
    println!("{}", map);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let lights: Vec<Light> = parse(input)?.collect();
    let (_, time_to_answer) = find_min_area(lights);
    println!("time to answer: {}", time_to_answer);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Failed to parse input line as Light")]
    ParseError,
    #[error("Failed to parse a value as an integer")]
    ParseIntError(#[from] ParseIntError),
    #[error("No solution found")]
    NoSolution,
}
