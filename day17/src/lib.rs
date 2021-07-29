mod offset_map;

pub use offset_map::OffsetMap;

use aoclib::{
    geometry::{tile::DisplayWidth, Direction, Point},
    parse,
};
use std::{collections::VecDeque, path::Path};

const WATER_X: i32 = 500;

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

#[derive(Debug, Clone, Copy)]
struct Wavefront {
    position: Point,
    direction: Direction,
}

/// Fill the map from an infinite water source located at the given x position and `y==0`.
fn fill_with_water(water_x: i32, mut map: OffsetMap<Tile>) -> Result<OffsetMap<Tile>, Error> {
    if water_x < map.low_x() || water_x > map.high_x() {
        return Err(Error::WaterSourceOutOfBounds);
    }

    // The basic approach here is a set of cursors which follow the propagation
    // front of the water.
    //
    // At each step, a cursor wets the tile at its current position, and then
    // considers where to add successors. If the tile below is sand, it adds a
    // single child below it. If it's clay, it adds one (if it already has a
    // side direction) or two (if its direction was down).
    //
    // That's all, really. It loops until there are no more legal successors.

    let initial_point = Point::new(water_x, map.high_y());
    debug_assert!(map.in_bounds(initial_point));
    let mut wavefronts = VecDeque::new();
    wavefronts.push_back(Wavefront {
        position: initial_point,
        direction: Direction::Down,
    });

    while let Some(Wavefront {
        position,
        direction,
    }) = wavefronts.pop_front()
    {
        // wet the sand
        match map[position] {
            Tile::Sand => map[position] = Tile::WaterPassthrough,
            Tile::Water => {
                unreachable!("wavefront propagation should never work backwards")
            }
            // we've made it back to a known state, no need to do any more work here
            Tile::WaterPassthrough => continue,
            Tile::Clay => {
                // Clay is the complicated case, because when we've hit clay, we have to
                // project backwards to determine whether or not to fill this level with water
                let backwards = match direction {
                    Direction::Right | Direction::Left => direction.reverse(),
                    // if we've gone up and hit a roof, no problem and no successors
                    Direction::Up => continue,
                    Direction::Down => {
                        unreachable!("down tiles shouldn't be clay; filtered elsewhere")
                    }
                };
                let prev_point = position + backwards;
                let (dx, dy) = backwards.deltas();
                let mut should_fill = false;
                // walk backwards through the points in this row to determine if this is a fill situation.
                for point in map.project(prev_point, dx, dy) {
                    match map[point] {
                        // if sand, the other back-projection will do the job later
                        Tile::Sand => break,
                        // if clay, then this is a basin and should fill
                        Tile::Clay => {
                            should_fill = true;
                            break;
                        }
                        // passthroughs are uninteresting and common; do nothing
                        Tile::WaterPassthrough => {}
                        Tile::Water => unreachable!("water propagation should never be incomplete"),
                    }
                }

                // if we should fill, do it all again
                if should_fill {
                    let mut point = prev_point;
                    while map.in_bounds(point) {
                        match map[point] {
                            Tile::Clay => break,
                            Tile::WaterPassthrough => map[point] = Tile::Water,
                            Tile::Sand => unreachable!("guaranteed by previous iteration"),
                            Tile::Water => {
                                unreachable!("water propagation should never collide")
                            }
                        }
                        point += backwards;
                    }

                    // the other consequence of filling a row is that we add a child upwards of the current
                    // position, so we can fill above
                    wavefronts.push_back(Wavefront {
                        position: position + Direction::Up,
                        direction: Direction::Up,
                    });
                    // note: we need to rearchitect this WHOLE THING: this is buggy
                    // otherwise, we're not following the no-water-pressure rule: a wall dipping into the
                    // pool will potentially fill the wrong side. How can we ensure we
                    todo!()
                }
            }
        }

        // decide where to propagate
        // everyone goes down first if possible
        let down = position + Direction::Down;
        if let Tile::Sand = map[down] {
            wavefronts.push_back(Wavefront {
                position: down,
                direction: Direction::Down,
            });
            // if we can flow down, we do not also flow sideways
            continue;
        }

        for sideways_direction in [Direction::Left, Direction::Right] {
            if direction == sideways_direction || direction == Direction::Down {
                let successor = position + sideways_direction;
                debug_assert!(
                    map.in_bounds(successor),
                    "water must not flow over the edge"
                );
                wavefronts.push_back(Wavefront {
                    position: successor,
                    direction: sideways_direction,
                });
            }
        }
    }

    Ok(map)
}

pub fn part1(input: &Path, show_map: bool) -> Result<(), Error> {
    let veins: Vec<Vein> = parse(input)?.collect();
    let map = OffsetMap::new(&veins);
    let map = fill_with_water(WATER_X, map)?;
    let wet_tiles = map
        .iter()
        .filter(|tile| matches!(*tile, Tile::WaterPassthrough | Tile::Water))
        .count();
    println!("n wet tiles: {}", wet_tiles);
    if show_map {
        println!("{}", map);
    }
    Ok(())
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Water source does not intercept known clay deposits")]
    WaterSourceOutOfBounds,
    #[error("Water flowed over map edge during calculation")]
    WaterFlowedOverEdge,
}
