use crate::{Error, Tile, Unit, Units};
use std::{
    ops::{Deref, Index},
    path::Path,
    str::FromStr,
};

type InnerMap = aoclib::geometry::Map<Tile>;

#[derive(Clone)]
pub(crate) struct Map(pub(crate) InnerMap);

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        InnerMap::try_from(std::io::Cursor::new(s))
            .map(Map)
            .map_err(Into::into)
    }
}

impl Deref for Map {
    type Target = InnerMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I> Index<I> for Map
where
    InnerMap: Index<I>,
{
    type Output = <InnerMap as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl Map {
    pub fn load(input: &Path) -> Result<Self, Error> {
        std::fs::read_to_string(input)?.parse()
    }

    /// Extract the units from this map into their own data structure,
    /// leaving only the immovable tiles of the map.
    pub fn units(&mut self) -> Units {
        let mut units = Vec::new();
        self.0.iter_mut().for_each(|(position, tile)| {
            if let Tile::Occupied(unit_type) = *tile {
                *tile = Tile::Empty;
                units.push(Unit::new(unit_type, position));
            }
        });
        Units { map: self, units }
    }
}
