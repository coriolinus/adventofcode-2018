



use day04::Record;
use failure::Error;
use util::get_input_lines_as;

fn main() -> Result<(), Error> {
    let mut records = get_input_lines_as::<Record>()?;
    records.sort_unstable();

    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();
    Record::debug(&mut stdout_lock, &records);

    Ok(())
}
