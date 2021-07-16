use enum_iterator::IntoEnumIterator;
use pest_consume::{match_nodes, Parser};
use std::{convert::TryInto, path::Path, str::FromStr};

#[derive(Parser)]
#[grammar = "parser.pest"]
struct InputParser;

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
            [samples(samples), example_program(_example_program), EOI(_eoi)] => Input { samples, _example_program }
        ))
    }
}

impl InputParser {
    fn parse_str(input: &str) -> ParseResult<Input> {
        let input = Self::parse(Rule::input, input)?.single()?;
        Self::input(input)
    }

    fn parse_file(path: &Path) -> Result<Input, Error> {
        let input = std::fs::read_to_string(path)?;
        Self::parse_str(&input).map_err(Into::into)
    }
}

struct Input {
    samples: Vec<Sample>,
    _example_program: Vec<UnknownInstruction>,
}

type Value = u32;

/// Opcodes control the behavior of an instruction and how the inputs are interpreted.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    parse_display::FromStr,
    parse_display::Display,
    IntoEnumIterator,
)]
#[display(style = "lowercase")]
enum Opcode {
    // Addition
    Addr,
    Addi,
    // Multiplication
    Mulr,
    Muli,
    // Bitwise And
    Banr,
    Bani,
    // Bitwise Or
    Borr,
    Bori,
    // Assignment
    Setr,
    Seti,
    // Greater-than testing
    Gtir,
    Gtri,
    Gtrr,
    // Equality testing
    Eqir,
    Eqri,
    Eqrr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Instruction {
    opcode: Opcode,
    a: Value,
    b: Value,
    c: Value,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    parse_display::FromStr,
    parse_display::Display,
)]
#[display("{opcode} {a} {b} {c}")]
struct UnknownInstruction {
    opcode: Value,
    a: Value,
    b: Value,
    c: Value,
}

impl UnknownInstruction {
    fn assume(self, opcode: Opcode) -> Instruction {
        let UnknownInstruction { a, b, c, .. } = self;
        Instruction { opcode, a, b, c }
    }
}

type Registers = [Value; 4];

struct Cpu {
    registers: Registers,
}

impl Cpu {
    fn new(registers: Registers) -> Self {
        Self { registers }
    }

    fn register(&self, index: Value) -> Result<&Value, CpuError> {
        self.registers
            .get(index as usize)
            .ok_or(CpuError::InvalidRegister)
    }

    fn register_mut(&mut self, index: Value) -> Result<&mut Value, CpuError> {
        self.registers
            .get_mut(index as usize)
            .ok_or(CpuError::InvalidRegister)
    }

    fn execute(&mut self, instruction: Instruction) -> Result<(), CpuError> {
        use Opcode::*;

        let value = match instruction.opcode {
            Addr => self.register(instruction.a)? + self.register(instruction.b)?,
            Addi => self.register(instruction.a)? + instruction.b,
            Mulr => self.register(instruction.a)? * self.register(instruction.b)?,
            Muli => self.register(instruction.a)? * instruction.b,
            Banr => self.register(instruction.a)? & self.register(instruction.b)?,
            Bani => self.register(instruction.a)? & instruction.b,
            Borr => self.register(instruction.a)? | self.register(instruction.b)?,
            Bori => self.register(instruction.a)? | instruction.b,
            Setr => *self.register(instruction.a)?,
            Seti => instruction.a,
            Gtir => {
                if instruction.a > *self.register(instruction.b)? {
                    1
                } else {
                    0
                }
            }
            Gtri => {
                if *self.register(instruction.a)? > instruction.b {
                    1
                } else {
                    0
                }
            }
            Gtrr => {
                if *self.register(instruction.a)? > *self.register(instruction.b)? {
                    1
                } else {
                    0
                }
            }
            Eqir => {
                if instruction.a == *self.register(instruction.b)? {
                    1
                } else {
                    0
                }
            }
            Eqri => {
                if *self.register(instruction.a)? == instruction.b {
                    1
                } else {
                    0
                }
            }
            Eqrr => {
                if *self.register(instruction.a)? == *self.register(instruction.b)? {
                    1
                } else {
                    0
                }
            }
        };
        *self.register_mut(instruction.c)? = value;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct Sample {
    before: Registers,
    unknown_instruction: UnknownInstruction,
    after: Registers,
}

impl Sample {
    fn behaves_like(self) -> impl Iterator<Item = Opcode> {
        Opcode::into_enum_iter().filter_map(move |opcode| {
            let instruction = self.unknown_instruction.assume(opcode);
            let mut cpu = Cpu::new(self.before.into());
            cpu.execute(instruction).ok()?;
            let after: [Value; 4] = self.after.into();
            (cpu.registers == after).then(move || opcode)
        })
    }
}

impl FromStr for Sample {
    type Err = pest_consume::Error<Rule>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = InputParser::parse(Rule::sample, s)?.single()?;
        InputParser::sample(s)
    }
}

#[derive(Debug, thiserror::Error)]
enum CpuError {
    #[error("requested a register which does not exist")]
    InvalidRegister,
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let input = InputParser::parse_file(input)?;
    let samples_with_at_lest_three_possibilities = input
        .samples
        .iter()
        .filter(|sample| sample.behaves_like().count() >= 3)
        .count();
    println!(
        "samples with at least three possibilities: {}",
        samples_with_at_lest_three_possibilities
    );
    Ok(())
}

pub fn part2(_input: &Path) -> Result<(), Error> {
    unimplemented!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("parse error")]
    Parse(#[from] pest_consume::Error<Rule>),
    #[error("No solution found")]
    NoSolution,
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashset;
    use std::collections::HashSet;

    const EXAMPLE_SAMPLE: &str = "Before: [3, 2, 1, 1]
9 2 1 2
After:  [3, 2, 2, 1]";

    #[test]
    fn test_example() {
        let sample: Sample = EXAMPLE_SAMPLE.parse().unwrap();
        let expect = hashset! {Opcode::Mulr, Opcode::Addi, Opcode::Seti};
        let have: HashSet<_> = sample.behaves_like().collect();
        assert_eq!(expect, have);
    }
}
