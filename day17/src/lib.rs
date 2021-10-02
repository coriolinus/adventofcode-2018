use aoclib::{
    geometry::{
        tile::{DisplayWidth, ToRgb},
        Direction, Point,
    },
    parse,
};
use std::{
    collections::VecDeque,
    convert::TryInto,
    path::{Path, PathBuf},
    rc::Rc,
};

#[cfg(feature = "animate")]
use {aoclib::geometry::map::Style, std::time::Duration};

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

impl Tile {
    pub fn is_dry(&self) -> bool {
        matches!(*self, Tile::Sand | Tile::Clay)
    }

    pub fn is_wet(&self) -> bool {
        !self.is_dry()
    }
}

impl DisplayWidth for Tile {
    const DISPLAY_WIDTH: usize = 1;
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Sand
    }
}

impl ToRgb for Tile {
    fn to_rgb(&self) -> [u8; 3] {
        match self {
            Tile::Sand => [250, 231, 180],
            Tile::Clay => [94, 85, 59],
            Tile::WaterPassthrough => [224, 255, 252],
            Tile::Water => [54, 88, 181],
        }
    }
}

type Map = aoclib::geometry::Map<Tile>;

fn make_map(veins: &[Vein]) -> Map {
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

    // adjust the x values to provide one tile of margin at the sides
    // this ensures that we never fail to account for some water flow
    min.x -= 1;
    max.x += 1;

    let width = (max.x - min.x + 1) as usize;
    let height = (max.y - min.y + 1) as usize;
    let offset = min;

    let mut map = Map::new_offset(offset, width, height);
    for vein in veins {
        for point in vein.points() {
            map[point] = crate::Tile::Clay;
        }
    }

    // AoC is upside down
    map.flip_vertical()
}

#[derive(Debug, Clone)]
struct Wavefront {
    position: Point,
    prev_point: Option<Point>,
    backtrack: Option<Rc<Wavefront>>,
}

/// Fill the map from an infinite water source located at the given x position and max `y`.
fn fill_with_water(
    water_x: i32,
    mut map: Map,
    animation_path: Option<PathBuf>,
) -> Result<Map, Error> {
    if water_x < map.low_x() || water_x > map.high_x() {
        return Err(Error::WaterSourceOutOfBounds);
    }

    #[cfg(not(feature = "animate"))]
    if animation_path.is_some() {
        return Err(Error::MissingFeature);
    }

    #[cfg(feature = "animate")]
    let mut animation = animation_path.and_then(|path| {
        map.prepare_animation(&path, Duration::from_millis(300), Style::Fill)
            .ok()
    });

    macro_rules! frame {
        () => {
            #[cfg(feature = "animate")]
            if let Some(ref mut animation) = animation {
                animation.write_frame(&map)?;
            }
        };
    }

    // write initial frames
    for _ in 0..6 {
        frame!();
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
        prev_point: None,
        backtrack: None,
    });

    while let Some(wavefront) = wavefronts.pop_front() {
        frame!();
        let wavefront = Rc::new(wavefront);

        // wet the sand
        match map[wavefront.position] {
            Tile::Sand => map[wavefront.position] = Tile::WaterPassthrough,
            Tile::Water => {
                unreachable!("wavefront propagation should never work backwards")
            }
            // we've made it back to a known state, no need to do any more work here
            Tile::WaterPassthrough => continue,
            Tile::Clay => {
                // Clay is complicated so we've factored it out
                handle_clay(&mut map, &wavefront, &mut wavefronts);

                // in any case, clay tiles have no normal successors
                continue;
            }
        }

        // decide where to propagate
        // everyone goes down first if possible
        let down = wavefront.position + Direction::Down;
        if map.in_bounds(down) {
            if let Tile::Sand = map[down] {
                wavefronts.push_back(Wavefront {
                    position: down,
                    prev_point: Some(wavefront.position),
                    backtrack: Some(wavefront.clone()),
                });
                // if we can flow down, we do not also flow sideways
                continue;
            }
        } else {
            // if we've flowed down past the bottom of the map, we must not follow
            // up by flowing sideways. It's assumed sand, not clay.
            continue;
        }

        for sideways_direction in [Direction::Left, Direction::Right] {
            let successor = wavefront.position + sideways_direction;
            debug_assert!(
                map.in_bounds(successor),
                "water must not flow over the edge"
            );
            if map[successor].is_dry() {
                wavefronts.push_back(Wavefront {
                    position: successor,
                    prev_point: Some(wavefront.position),
                    backtrack: wavefront.backtrack.clone(),
                });
            }
        }
    }

    // write final frames
    for _ in 0..4 {
        frame!();
    }

    Ok(map)
}

// Clay is the complicated case, because when we've hit clay, we have to
// project backwards to determine whether or not to fill this level with water
fn handle_clay(map: &mut Map, wavefront: &Rc<Wavefront>, wavefronts: &mut VecDeque<Wavefront>) {
    if let Some(prev_point) = wavefront.prev_point {
        let direction: Direction = (wavefront.position - prev_point)
            .try_into()
            .expect("wavefront always propogates to orthogonal adjacent");
        debug_assert_ne!(direction, Direction::Up, "we never move upwards");
        debug_assert_ne!(direction, Direction::Down, "we never move down into clay");

        let backwards = direction.reverse();
        let (dx, dy) = backwards.deltas();
        let mut should_fill = false;
        // walk backwards through the points in this row to determine if this is a fill situation.
        for point in map.project(prev_point, dx, dy) {
            match map[point] {
                // passthroughs are uninteresting and common; do nothing
                Tile::WaterPassthrough => {}
                // if sand, the other back-projection will do the job later
                Tile::Sand => break,
                // if clay, then this is a basin and should fill
                Tile::Clay => {
                    should_fill = true;
                    break;
                }
                // we can reach water if, earlier this tick, the other side of the basin
                // was detected, and filled this entire row with water. In that case,
                // we don't need to do anything in particular.
                Tile::Water => {
                    break;
                }
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

            // the other consequence of filling a row is that we add children adjacent to the backtrack position
            if let Some(ref backtrack) = wavefront.backtrack {
                for direction in [Direction::Left, Direction::Right] {
                    wavefronts.push_back(Wavefront {
                        position: backtrack.position + direction,
                        prev_point: Some(backtrack.position),
                        backtrack: backtrack.backtrack.clone(),
                    });
                }
            }
        }
    }
}

// known wrong, too low: 1226
pub fn part1(input: &Path, show_map: bool, animation_path: Option<PathBuf>) -> Result<(), Error> {
    let veins: Vec<Vein> = parse(input)?.collect();
    let map = make_map(&veins);
    let map = fill_with_water(WATER_X, map, animation_path)?;
    let wet_tiles = map.iter().filter(|(_point, tile)| tile.is_wet()).count();
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
    #[error("You set an animation path but did not compile with 'animation' feature")]
    MissingFeature,
    #[cfg(feature = "animate")]
    #[error("encoding animation")]
    EncodingAnimation(#[from] aoclib::geometry::map::EncodingError),
}
