use text_io;

use std::str::FromStr;
use text_io::try_scan;

/// a coordinate pair: `(x, y)`
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Coords(usize, usize);

impl FromStr for Coords {
    type Err = text_io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y): (usize, usize);
        try_scan!(s.bytes() => "{}, {}", x, y);
        Ok(Coords(x, y))
    }
}

impl Coords {
    pub fn manhattan(self, other: Self) -> usize {
        use std::cmp::{max, min};

        max(self.0, other.0) - min(self.0, other.0) + max(self.1, other.1) - min(self.1, other.1)
    }
}

pub type PointID = usize;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    Empty,
    Point(PointID),
    Region(PointID),
    Equidistant,
    Safe,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

impl Into<char> for Cell {
    fn into(self) -> char {
        match self {
            Cell::Empty => '∅',
            Cell::Equidistant => '.',
            Cell::Point(idx) => (idx as u8 + 'A' as u8) as char,
            Cell::Region(idx) => (idx as u8 + 'a' as u8) as char,
            Cell::Safe => '#',
        }
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <Self as Into<char>>::into(*self))
    }
}

pub type Field = Vec<Vec<Cell>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Site(PointID),
    Circle,
}

pub fn field_debug(f: &Field) -> String {
    let ss = f
        .iter()
        .map(|row| {
            row.iter()
                .map(|&cell| <Cell as Into<char>>::into(cell))
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    ss.join("\n")
}

// naive `n^3` implementation
fn make_field<Filter, Transform>(
    points: &[Coords],
    filter: Filter,
    transform: Transform,
) -> Option<Field>
where
    Filter: Fn(Cell) -> bool,
    Transform: Fn(Coords, &[Coords]) -> Cell,
{
    if points.is_empty() {
        return None;
    }
    let max_x = points.iter().map(|p| p.0).max()?;
    let max_y = points.iter().map(|p| p.1).max()?;

    let mut field = vec![vec![Cell::default(); max_x + 1]; max_y + 1];

    for (idx, Coords(x, y)) in points.iter().enumerate() {
        field[*y][*x] = Cell::Point(idx);
    }

    for y in 0..=max_y {
        for x in 0..=max_x {
            // we only want to edit empty cells
            #[cfg(feature = "debug_out")]
            {
                let c: char = field[y][x].into();
                print!("({}, {}): {}", x, y, c);
            }

            if filter(field[y][x]) {
                field[y][x] = transform(Coords(x, y), points);

                #[cfg(feature = "debug_out")]
                {
                    let c: char = field[y][x].into();
                    print!(" => {}", c);
                }
            }
            #[cfg(feature = "debug_out")]
            println!("");
        }
    }

    #[cfg(feature = "debug_out")]
    println!("{}", field_debug(&field));

    Some(field)
}

pub fn voronoi(points: &[Coords]) -> Option<Field> {
    make_field(
        points,
        |cell| cell == Cell::Empty,
        |Coords(x, y), points| {
            if points.len() < 2 {
                return Cell::Region(0);
            }

            let mut dists: Vec<_> = points
                .iter()
                .enumerate()
                .map(|(idx, &p)| (Coords(x, y).manhattan(p), idx))
                .collect();
            dists.sort();

            #[cfg(feature = "debug_out")]
            print!(" -> [{:?}, {:?}, ...]", dists[0], dists[1]);

            if dists[0].0 == dists[1].0 {
                Cell::Equidistant
            } else {
                Cell::Region(dists[0].1)
            }
        },
    )
}

pub fn undermax(points: &[Coords], max_dist: usize) -> Option<Field> {
    make_field(
        points,
        |cell| match cell {
            Cell::Empty | Cell::Point(_) => true,
            _ => false,
        },
        |coords, points| {
            if points
                .iter()
                .map(|point| coords.manhattan(*point))
                .sum::<usize>()
                < max_dist
            {
                Cell::Safe
            } else {
                Cell::Empty
            }
        },
    )
}

/// returns the size of the largest finite region
pub fn biggest_finite(points: &[Coords], field: &Field) -> Option<u64> {
    let mut is_finite = vec![true; points.len()];
    let mut count = vec![0_u64; points.len()];

    for (y_idx, row) in field.iter().enumerate() {
        for (x_idx, cell) in row.iter().enumerate() {
            match cell {
                Cell::Empty => panic!("field must not contain empty cells"),
                Cell::Equidistant | Cell::Safe => (),
                Cell::Point(pid) | Cell::Region(pid) => {
                    if is_finite[*pid] {
                        count[*pid] += 1;
                        if y_idx == 0
                            || y_idx == field.len() - 1
                            || x_idx == 0
                            || x_idx == row.len() - 1
                        {
                            is_finite[*pid] = false;
                        }
                    }
                }
            }
        }
    }

    is_finite
        .iter()
        .zip(count)
        .filter_map(|(finite, count)| if *finite { Some(count) } else { None })
        .max()
}

pub fn safezone_size(field: &Field) -> usize {
    field
        .iter()
        .map(|row| row.iter().filter(|cell| **cell == Cell::Safe).count())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> Vec<Coords> {
        vec![
            Coords(1, 1),
            Coords(1, 6),
            Coords(8, 3),
            Coords(3, 4),
            Coords(5, 5),
            Coords(8, 9),
        ]
    }

    #[test]
    fn test_biggest_finite() {
        let points = example();
        let field = voronoi(&points).unwrap();
        let bf = biggest_finite(&points, &field).unwrap();
        assert_eq!(17, bf);
    }

    #[test]
    fn test_safezone_size() {
        let points = example();
        let field = undermax(&points, 32).unwrap();
        let ss = safezone_size(&field);
        assert_eq!(16, ss);
    }
}
