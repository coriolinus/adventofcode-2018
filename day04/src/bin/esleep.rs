use day04;



use day04::Record;
use failure::Error;
use util::get_input_lines_as;

fn main() -> Result<(), Error> {
    let mut records = get_input_lines_as::<Record>()?;
    records.sort_unstable();

    println!(
        "{:>14}: {:?}",
        "earliest sleep",
        records
            .iter()
            .filter(|r| r.event == day04::Event::FallsAsleep)
            .map(|r| (r.timestamp.time(), r))
            .min()
            .map(|(_, r)| r.timestamp)
    );

    println!(
        "{:>14}: {:?}",
        "latest wake",
        records
            .iter()
            .filter(|r| r.event == day04::Event::WakesUp)
            .map(|r| (r.timestamp.time(), r))
            .max()
            .map(|(_, r)| r.timestamp)
    );

    Ok(())
}
