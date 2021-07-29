mod offset_map;

pub use offset_map::OffsetMap;

use aoclib::{
    geometry::{tile::DisplayWidth, Point},
    parse,
};
use std::path::Path;

#[derive(Debug, Clone, Copy, parse_display::FromStr, parse_display::Display)]
enum Vein {
    #[display("x={x}, y={y_min}..{y_max}")]
    Vertical { x: i32, y_min: i32, y_max: i32 },
    #[display("y={y}, x={x_min}..{x_max}")]
    Horizontal { x_min: i32, x_max: i32, y: i32 },
}

impl Vein {
    fn points(self) -> Box<dyn Iterator<Item = Point>> {
        match self {
            Self::Vertical { x, y_min, y_max } => {
                Box::new((y_min..=y_max).map(move |y| Point::new(x, y)))
            }
            Self::Horizontal { x_min, x_max, y } => {
                Box::new((x_min..=x_max).map(move |x| Point::new(x, y)))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, parse_display::FromStr, parse_display::Display)]
pub(crate) enum Tile {
    #[display(".")]
    Sand,
    #[display("#")]
    Clay,
    #[display("|")]
    WaterPassthrough,
    #[display("~")]
    Water,
}

impl DisplayWidth for Tile {
    const DISPLAY_WIDTH: usize = 1;
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Sand
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let veins: Vec<Vein> = parse(input)?.collect();
    let map = OffsetMap::new(&veins);
    println!("{}", map);
    unimplemented!()
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("No solution found")]
    NoSolution,
}
