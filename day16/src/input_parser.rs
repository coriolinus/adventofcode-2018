use crate::{Error, Input, Sample, UnknownInstruction, Value};
use pest_consume::{match_nodes, Parser};
use std::{convert::TryInto, path::Path};

#[derive(Parser)]
#[grammar = "parser.pest"]
pub struct InputParser;

type ParseResult<T> = Result<T, pest_consume::Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[pest_consume::parser]
impl InputParser {
    fn EOI(_input: Node) -> ParseResult<()> {
        Ok(())
    }

    fn number(input: Node) -> ParseResult<Value> {
        input.as_str().parse::<Value>().map_err(|e| input.error(e))
    }

    fn registers(input: Node) -> ParseResult<[Value; 4]> {
        Ok(match_nodes!(input.into_children();
            [number(numbers)..] => numbers.collect::<Vec<_>>()
                .try_into()
                .expect("pest guarantees we have four numbers here")
        ))
    }

    fn instruction(input: Node) -> ParseResult<UnknownInstruction> {
        Ok(match_nodes!(input.into_children();
            [number(numbers)..] => {
                let [opcode, a, b, c]: [Value; 4] = numbers.collect::<Vec<_>>()
                    .try_into()
                    .expect("pest guarantees we have four numbers here");
                UnknownInstruction { opcode, a, b, c }
            }
        ))
    }

    fn sample(input: Node) -> ParseResult<Sample> {
        Ok(match_nodes!(input.into_children();
            [registers(before), instruction(unknown_instruction), registers(after)] => {
                Sample { before, unknown_instruction, after }
            }
        ))
    }

    fn samples(input: Node) -> ParseResult<Vec<Sample>> {
        Ok(match_nodes!(input.into_children();
            [sample(samples)..] => samples.collect()
        ))
    }

    fn example_program(input: Node) -> ParseResult<Vec<UnknownInstruction>> {
        Ok(match_nodes!(input.into_children();
            [instruction(instructions)..] => instructions.collect()
        ))
    }

    fn input(input: Node) -> ParseResult<Input> {
        Ok(match_nodes!(input.into_children();
            [samples(samples), example_program(example_program), EOI(_eoi)] => Input { samples, example_program }
        ))
    }
}

impl InputParser {
    pub(crate) fn parse_str(input: &str) -> ParseResult<Input> {
        let input = Self::parse(Rule::input, input)?.single()?;
        Self::input(input)
    }

    pub(crate) fn parse_file(path: &Path) -> Result<Input, Error> {
        let input = std::fs::read_to_string(path)?;
        Self::parse_str(&input).map_err(Into::into)
    }

    pub(crate) fn parse_sample(s: &str) -> ParseResult<Sample> {
        let s = InputParser::parse(Rule::sample, s)?.single()?;
        InputParser::sample(s)
    }
}
