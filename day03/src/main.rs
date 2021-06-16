



use day01::get_input_lines_as;
use day03::{apply_claims, Claim, find_uncontended};
use failure::Error;

fn main() -> Result<(), Error> {
    let lines = get_input_lines_as::<Claim>()?;
    println!("parsed {} input lines", lines.len());

    let field = apply_claims(&lines);
    let contended: usize = field
        .iter()
        .map(|row| row.iter().filter(|x| x > &&1).count())
        .sum();
    println!("{} sq inches contended", contended);
    println!("uncontended claim: {}", find_uncontended(&lines, &field));
    Ok(())
}
