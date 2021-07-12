use aoclib::{geometry::Point as LibPoint, parse};
use std::path::Path;

#[derive(Debug, parse_display::FromStr, parse_display::Display)]
#[display("{x}, {y}")]
pub struct Point {
    x: i32,
    y: i32,
}

impl From<Point> for LibPoint {
    fn from(Point { x, y }: Point) -> Self {
        LibPoint::new(x, y)
    }
}

pub fn parse_points(input: &Path) -> std::io::Result<Vec<LibPoint>> {
    parse::<Point>(input).map(|iter| iter.map(Into::into).collect())
}
