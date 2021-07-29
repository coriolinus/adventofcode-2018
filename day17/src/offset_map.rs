//! Module for `OffsetMap`, which is a map whose origin is not `(0, 0)`.
//
// Should strongly consider extracting this into `aoclib` in the future.
// It'll require some non-trivial implementation to ensure that the interface
// is the same, but it's proven to be a useful interface.
//
// A somewhat better idea: just update standard `Map` with offset-aware methods.

use crate::Vein;
use aoclib::geometry::{tile::DisplayWidth, Map, Point};
use std::{
    fmt::{self},
    ops::{Index, IndexMut},
};

/// A `Map` whose origin is not necessarily at `(0, 0)`.
///
/// This can significantly reduce storage / display requirements for
/// sparse maps distant from the origin.
pub struct OffsetMap<Tile> {
    offset: Point,
    map: Map<Tile>,
}

impl OffsetMap<crate::Tile> {
    pub fn new(veins: &[Vein]) -> Self {
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
                map[point - offset] = crate::Tile::Clay;
            }
        }

        OffsetMap { offset, map }
    }
}

impl<Tile> OffsetMap<Tile> {
    pub fn width(&self) -> usize {
        self.map.width()
    }

    pub fn height(&self) -> usize {
        self.map.height()
    }

    pub fn offset(&self) -> Point {
        self.offset
    }

    pub fn low_x(&self) -> i32 {
        self.offset.x
    }

    pub fn high_x(&self) -> i32 {
        self.offset.x + self.width() as i32
    }

    pub fn low_y(&self) -> i32 {
        self.offset.y
    }

    pub fn high_y(&self) -> i32 {
        self.offset.y + self.height() as i32
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tile> {
        self.map.iter()
    }
}

impl<Tile> Index<Point> for OffsetMap<Tile> {
    type Output = Tile;

    fn index(&self, index: Point) -> &Self::Output {
        self.map.index(index - self.offset)
    }
}

impl<Tile> IndexMut<Point> for OffsetMap<Tile> {
    fn index_mut(&mut self, index: Point) -> &mut Self::Output {
        self.map.index_mut(index - self.offset)
    }
}

impl<Tile> fmt::Display for OffsetMap<Tile>
where
    Tile: fmt::Display + DisplayWidth + Clone + Default,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // AoC origin is in upper left, not lower left
        let map = self.map.flip_vertical();
        write!(f, "{}", map)
    }
}
