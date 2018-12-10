use failure::Fail;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;
use std::num::ParseIntError;
use std::ops::Add;
use std::str::FromStr;

lazy_static! {
    static ref POINT_RE: Regex =
        Regex::new(r"position=(?P<position><\s?(?P<px>-?\d+), \s?(?P<py>-?\d+)>) velocity=(?P<velocity><\s?(?P<vx>-?\d+), \s?(?P<vy>-?\d+)>)").unwrap();
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Vec2 {
    x: i64,
    y: i64,
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Self) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{:2}, {:2}>", self.x, self.y)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    position: Vec2,
    velocity: Vec2,
}

#[derive(Debug, Fail)]
pub enum ParsePointError {
    #[fail(display = "capture not found")]
    CaptureNotFound,
    #[fail(display = "parse int error: {}", _0)]
    ParseInt(#[cause] ParseIntError),
}

impl From<ParseIntError> for ParsePointError {
    fn from(err: ParseIntError) -> ParsePointError {
        ParsePointError::ParseInt(err)
    }
}

impl FromStr for Point {
    type Err = ParsePointError;

    fn from_str(s: &str) -> Result<Point, Self::Err> {
        let captures = POINT_RE
            .captures(s)
            .ok_or(ParsePointError::CaptureNotFound)?;
        let px = captures
            .name("px")
            .ok_or(ParsePointError::CaptureNotFound)?
            .as_str()
            .parse()?;
        let py = captures
            .name("py")
            .ok_or(ParsePointError::CaptureNotFound)?
            .as_str()
            .parse()?;
        let vx = captures
            .name("vx")
            .ok_or(ParsePointError::CaptureNotFound)?
            .as_str()
            .parse()?;
        let vy = captures
            .name("vy")
            .ok_or(ParsePointError::CaptureNotFound)?
            .as_str()
            .parse()?;

        Ok(Point {
            position: Vec2 { x: px, y: py },
            velocity: Vec2 { x: vx, y: vy },
        })
    }
}

fn bounds(points: &[Point]) -> Option<(Vec2, Vec2)> {
    let mut min_x = None;
    let mut max_x = None;
    let mut min_y = None;
    let mut max_y = None;

    for point in points {
        min_x = match min_x {
            Some(cmin_x) if cmin_x < point.position.x => Some(cmin_x),
            _ => Some(point.position.x),
        };
        min_y = match min_y {
            Some(cmin_y) if cmin_y < point.position.y => Some(cmin_y),
            _ => Some(point.position.y),
        };
        max_x = match max_x {
            Some(cmax_x) if cmax_x > point.position.x => Some(cmax_x),
            _ => Some(point.position.x),
        };
        max_y = match max_y {
            Some(cmax_y) if cmax_y > point.position.y => Some(cmax_y),
            _ => Some(point.position.y),
        };
    }

    Some((
        Vec2 {
            x: min_x?,
            y: min_y?,
        },
        Vec2 {
            x: max_x?,
            y: max_y?,
        },
    ))
}

pub fn area(points: &[Point]) -> Option<i64> {
    let (min, max) = bounds(points)?;
    Some(((max.x - min.x) * (max.y - min.y)).abs())
}

pub fn next_state(points: &[Point], next: &mut [Point]) {
    for (idx, point) in points.iter().enumerate() {
        next[idx] = Point {
            position: point.position + point.velocity,
            ..*point
        };
    }
}

pub fn find_min_area(points: &[Point]) -> (Vec<Point>, usize) {
    use std::mem::swap;
    let mut count = 0;

    let mut prev_state = points.to_vec();
    let mut state = prev_state.clone();

    while area(&state) <= area(&prev_state) {
        next_state(&mut state, &mut prev_state);
        swap(&mut prev_state, &mut state);
        count += 1;
    }

    count -= 1;

    (prev_state, count)
}

pub fn display(points: &[Point]) -> Option<String> {
    let (min, max) = bounds(points)?;

    let mut out = vec![vec!['.'; (max.x - min.x + 1) as usize]; (max.y - min.y + 1) as usize];

    for Point {
        position: Vec2 { x, y },
        ..
    } in points
    {
        let x_idx = (x - min.x) as usize;
        let y_idx = (y - min.y) as usize;

        out[y_idx][x_idx] = '#';
    }

    Some(out.iter().map(|row| row.iter().join("")).join("\n"))
}
