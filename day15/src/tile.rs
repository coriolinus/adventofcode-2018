use crate::{UnitPositions, UnitType};
use aoclib::geometry::{
    map::{ContextFrom, Traversable},
    tile::DisplayWidth,
    Point,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, parse_display::FromStr, parse_display::Display)]
pub(crate) enum Tile {
    #[display(".")]
    Empty,
    #[display("#")]
    Wall,
    #[display("{0}")]
    Occupied(UnitType),
}

impl DisplayWidth for Tile {
    const DISPLAY_WIDTH: usize = 1;
}

impl ContextFrom<Tile> for Traversable {
    type Context = UnitPositions;

    fn ctx_from(t: Tile, position: Point, context: &Self::Context) -> Self {
        match t {
            Tile::Empty => {
                if context.contains_key(&position) {
                    Traversable::Obstructed
                } else {
                    Traversable::Free
                }
            }
            Tile::Wall | Tile::Occupied(_) => Traversable::Obstructed,
        }
    }
}
