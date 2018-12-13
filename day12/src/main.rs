use day12::{Input, State};
use failure::{format_err, Error};

fn parse_input() -> Result<Input, Error> {
    let input = util::get_input_string()?;
    let parser = day12::input::InputParser::new();
    let input = parser
        .parse(&input)
        .map_err(|e| format_err!("{}", e.to_string()))?;
    Ok(input)
}

fn main() -> Result<(), Error> {
    let input = parse_input()?;
    println!("{} pots; {} rules", input.initial.len(), input.rules.len());

    const GENERATION_TARGET: usize = 20;
    let rules = input.rules();
    let mut state: State = input.initial.into();

    state = day12::steps(&state, &rules, GENERATION_TARGET);
    println!("idx sum: {}", state.sum_indices(GENERATION_TARGET));

    Ok(())
}
