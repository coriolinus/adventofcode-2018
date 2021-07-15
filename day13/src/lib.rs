use aoclib::geometry::{tile::DisplayWidth, Direction, Point};
use std::{cmp::Ordering, fmt, path::Path, str::FromStr};

#[derive(Debug, Clone, Copy)]
enum Track {
    Empty,
    Horizontal,
    Vertical,
    SlashCurve,
    BackslashCurve,
    Cross,
    Cart(Direction),
}

impl FromStr for Track {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            " " | "." => Self::Empty,
            "-" => Self::Horizontal,
            "|" => Self::Vertical,
            "/" => Self::SlashCurve,
            "\\" => Self::BackslashCurve,
            "+" => Self::Cross,
            "<" => Self::Cart(Direction::Left),
            ">" => Self::Cart(Direction::Right),
            "^" => Self::Cart(Direction::Up),
            "v" => Self::Cart(Direction::Down),
            _ => return Err(Error::UnexpectedInput(s.to_string())),
        })
    }
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Track::Empty => " ",
            Track::Horizontal => "-",
            Track::Vertical => "|",
            Track::SlashCurve => "/",
            Track::BackslashCurve => "\\",
            Track::Cross => "+",
            Track::Cart(Direction::Left) => "<",
            Track::Cart(Direction::Right) => ">",
            Track::Cart(Direction::Up) => "^",
            Track::Cart(Direction::Down) => "v",
        })
    }
}

impl DisplayWidth for Track {
    const DISPLAY_WIDTH: usize = 1;
}

impl Default for Track {
    fn default() -> Self {
        Self::Empty
    }
}

struct Map(aoclib::geometry::Map<Track>);

impl Map {
    fn load(input: &Path) -> Result<Self, Error> {
        let file = std::fs::File::open(input)?;
        let reader = std::io::BufReader::new(file);
        Ok(Self(aoclib::geometry::Map::try_from(reader)?))
    }

    fn extract_carts(&mut self) -> Carts {
        let mut carts = Vec::new();
        self.0.for_each_point_mut(|track, position| {
            if let Track::Cart(direction) = *track {
                *track = match direction {
                    Direction::Right | Direction::Left => Track::Horizontal,
                    Direction::Up | Direction::Down => Track::Vertical,
                };
                carts.push(Cart::new(direction, position));
            }
        });
        debug_assert!(!self.0.iter().any(|&track| matches!(track, Track::Cart(_))));

        Carts {
            map: self,
            carts,
            remove_collisions: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cart {
    direction: Direction,
    position: Point,
    next_turn: Turn,
    dead: bool,
}

impl Cart {
    fn new(direction: Direction, position: Point) -> Cart {
        Cart {
            direction,
            position,
            next_turn: Turn::default(),
            dead: false,
        }
    }
}

/// Carts are sorted first from top to bottom, then left to right, then by incidentals.
impl Ord for Cart {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position
            .y
            .cmp(&other.position.y)
            .reverse()
            .then_with(|| self.position.x.cmp(&other.position.x))
            .then_with(|| self.dead.cmp(&other.dead))
            .then_with(|| self.direction.deltas().cmp(&other.direction.deltas()))
            .then_with(|| self.next_turn.cmp(&other.next_turn))
    }
}

impl PartialOrd for Cart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Turn {
    Left,
    Straight,
    Right,
}

impl Turn {
    fn next(self) -> Self {
        match self {
            Turn::Left => Turn::Straight,
            Turn::Straight => Turn::Right,
            Turn::Right => Turn::Left,
        }
    }
}

impl Default for Turn {
    fn default() -> Self {
        Turn::Left
    }
}

#[derive(Clone)]
struct Carts<'a> {
    map: &'a Map,
    carts: Vec<Cart>,
    remove_collisions: bool,
}

impl<'a> Carts<'a> {
    /// Advance a cart along its direction of motion by one tick.
    ///
    /// Return `new_position`.
    fn advance(cart: &mut Cart, map: &Map) -> Point {
        cart.direction = match (cart.direction, map.0[cart.position]) {
            (_, Track::Empty) => unreachable!("cart cannot travel off the rails"),
            (_, Track::Cart(_)) => unreachable!("carts most not be on the map"),
            (Direction::Right | Direction::Left, Track::Vertical)
            | (Direction::Up | Direction::Down, Track::Horizontal) => {
                unreachable!("cart direction mismatches track direction")
            }
            (direction, Track::Horizontal | Track::Vertical) => direction,
            (direction, Track::SlashCurve) => match direction {
                Direction::Right => Direction::Up,
                Direction::Left => Direction::Down,
                Direction::Up => Direction::Right,
                Direction::Down => Direction::Left,
            },
            (direction, Track::BackslashCurve) => match direction {
                Direction::Right => Direction::Down,
                Direction::Left => Direction::Up,
                Direction::Up => Direction::Left,
                Direction::Down => Direction::Right,
            },
            (direction, Track::Cross) => {
                let direction = match cart.next_turn {
                    Turn::Left => direction.turn_left(),
                    Turn::Straight => direction,
                    Turn::Right => direction.turn_right(),
                };
                cart.next_turn = cart.next_turn.next();
                direction
            }
        };
        cart.position += cart.direction;
        cart.position
    }

    /// Advance the simulation by one step.
    ///
    /// If two carts crash, return the points where the crash occurred.
    fn tick(&mut self) -> Vec<Point> {
        self.carts.sort_unstable();
        let mut collisions = Vec::new();

        for idx in 0..self.carts.len() {
            let new_position = {
                let cart = &mut self.carts[idx];
                if cart.dead {
                    debug_assert!(collisions.contains(&cart.position));
                    continue;
                }
                Self::advance(cart, self.map)
            };

            // there should only be one dead cart at any given point, but it can't hurt to check all of them.
            // we have to collect these in advance to avoid a double-borrow of `self.carts`.
            let collision_indices: Vec<_> = self
                .carts
                .iter()
                .enumerate()
                .filter_map(move |(collision_idx, cart)| {
                    (idx != collision_idx && !cart.dead && cart.position == new_position)
                        .then(move || collision_idx)
                })
                .collect();

            for collision_index in collision_indices {
                self.carts[collision_index].dead = true;
                self.carts[idx].dead = true;
                collisions.push(new_position);
            }
        }

        // clean up the carts list to get rid of the dead
        if self.remove_collisions {
            let old_cart_count = self.carts.len();

            self.carts.retain(|cart| !cart.dead);

            if !collisions.is_empty() {
                debug_assert_eq!(
                    old_cart_count - self.carts.len(),
                    2 * collisions.len(),
                    "each collision must remove two carts"
                );
            }
        }

        collisions
    }

    /// Adjust a point's y orientation to put the implicit origin at the top, instead of the bottom.
    fn flip_y(&self, mut point: Point) -> Point {
        point.y = self.map.0.height() as i32 - 1 - point.y;
        point
    }

    /// Loop until a collision is produced. Return the point of impact.
    fn run_until_first_collision(&mut self) -> Point {
        let mut collisions;
        loop {
            collisions = self.tick();
            if !collisions.is_empty() {
                break;
            }
        }
        self.flip_y(collisions[0])
    }

    /// Loop until only one cart remains. Return the position of the final cart.
    fn run_until_last_cart(&mut self) -> Result<Point, Error> {
        self.remove_collisions = true;
        while self.carts.len() > 1 {
            self.tick();
        }
        if self.carts.is_empty() {
            return Err(Error::NoSolution);
        }
        Ok(self.flip_y(self.carts[0].position))
    }
}

impl<'a> fmt::Display for Carts<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = self.map.0.clone();
        for cart in self.carts.iter() {
            map[cart.position] = Track::Cart(cart.direction);
        }
        writeln!(f, "{}", map)
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let mut map = Map::load(input)?;
    let mut carts = map.extract_carts();
    let first_collision = carts.run_until_first_collision();

    println!(
        "first collision at {},{}",
        first_collision.x, first_collision.y
    );
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let mut map = Map::load(input)?;
    let mut carts = map.extract_carts();
    let last_cart = carts.run_until_last_cart()?;

    println!("last cart at {},{}", last_cart.x, last_cart.y);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("No solution found: all carts collided")]
    NoSolution,
    #[error("Unexpected input for Track: {0}")]
    UnexpectedInput(String),
    #[error(transparent)]
    MapConversion(#[from] aoclib::geometry::map::MapConversionErr),
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_PART2: &str = "
/>-<\\..
|   |..
| /<+-\\
| | | v
\\>+</ |
  |   ^
  \\<->/
";

    // trim off the leading newline
    fn example_part2() -> &'static str {
        &EXAMPLE_PART2[1..]
    }

    #[test]
    fn test_example_part2() {
        let mut map = Map(
            aoclib::geometry::Map::<Track>::try_from(std::io::Cursor::new(example_part2()))
                .unwrap(),
        );
        // we're going to run this simulation twice: once to show debug output, once to show correct
        // behavior of the actual user function
        let mut carts = map.extract_carts();
        let mut carts2 = carts.clone();
        carts.remove_collisions = true;

        eprintln!("{}", &carts);

        while carts.carts.len() > 1 {
            carts.tick();
            eprintln!("{}", &carts);
        }

        assert_eq!(carts.flip_y(carts.carts[0].position), Point::new(6, 4));
        assert_eq!(carts2.run_until_last_cart().unwrap(), Point::new(6, 4));
    }
}
