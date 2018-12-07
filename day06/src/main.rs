use day06::{biggest_finite, safezone_size, undermax, voronoi, Coords};
use failure::format_err;

fn main() -> Result<(), failure::Error> {
    let inputs = util::get_input_lines_as::<Coords>()?;
    let field = voronoi(&inputs).ok_or_else(|| format_err!("could not voronoi"))?;
    let bf =
        biggest_finite(&inputs, &field).ok_or_else(|| format_err!("could not find biggest"))?;
    println!("biggest finite: {}", bf);
    let field =
        undermax(&inputs, 10000).ok_or_else(|| format_err!("could not find values under max"))?;
    let ss = safezone_size(&field);
    println!("safezone size:  {}", ss);
    Ok(())
}
