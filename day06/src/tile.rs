use aoclib::geometry::tile::DisplayWidth;

#[derive(Clone, Copy, PartialEq, Eq, Debug, parse_display::Display)]
pub enum Tile {
    #[display(" ")]
    Empty,
    #[display("o")]
    Point(usize),
    #[display("R")]
    Region(usize),
    #[display("X")]
    Equidistant,
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Empty
    }
}

impl DisplayWidth for Tile {
    const DISPLAY_WIDTH: usize = 1;
}
