use aoclib::{
    geometry::{tile::DisplayWidth, Map, Point},
    parse,
};
use std::{
    fmt::{self, Display},
    ops::{Deref, Index, IndexMut},
    path::Path,
};

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
enum Tile {
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

struct OffsetMap {
    offset: Point,
    map: Map<Tile>,
}

impl OffsetMap {
    fn new(veins: &[Vein]) -> Self {
        let mut min = Point::new(i32::MAX, i32::MAX);
        let mut max = Point::new(i32::MIN, i32::MIN);

        for vein in veins {
            for point in vein.points() {
                min.x = min.x.min(point.x);
                min.y = min.y.min(point.y);
                max.x = max.x.max(point.x);
                max.y = max.y.max(point.y);
            }
        }

        debug_assert!(min.x <= max.x);
        debug_assert!(min.y <= max.y);

        let width = (max.x - min.x + 1) as usize;
        let height = (max.y - min.y + 1) as usize;
        let offset = min;

        let mut map = Map::new(width, height);
        for vein in veins {
            for point in vein.points() {
                map[point - offset] = Tile::Clay;
            }
        }

        OffsetMap { offset, map }
    }
}

impl Deref for OffsetMap {
    type Target = Map<Tile>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl Index<Point> for OffsetMap {
    type Output = Tile;

    fn index(&self, index: Point) -> &Self::Output {
        self.map.index(index - self.offset)
    }
}

impl IndexMut<Point> for OffsetMap {
    fn index_mut(&mut self, index: Point) -> &mut Self::Output {
        self.map.index_mut(index - self.offset)
    }
}

impl fmt::Display for OffsetMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "offset: ({}, {})", self.offset.x, self.offset.y)?;
        writeln!(f, "dimentions: ({}, {})", self.width(), self.height())?;
        // AoC origin is in upper left, not lower left
        let map = self.map.flip_vertical();
        write!(f, "{}", map)
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
