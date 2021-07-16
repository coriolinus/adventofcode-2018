mod input_parser;

use enum_iterator::IntoEnumIterator;
use input_parser::InputParser;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    str::FromStr,
};

struct Input {
    samples: Vec<Sample>,
    example_program: Vec<UnknownInstruction>,
}

type Value = u32;

/// Opcodes control the behavior of an instruction and how the inputs are interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, IntoEnumIterator)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    fn assume_with(self, map: &HashMap<Value, Opcode>) -> Instruction {
        let opcode = map[&self.opcode];
        self.assume(opcode)
    }
}

type Registers = [Value; 4];

#[derive(Default, Debug)]
struct Cpu {
    registers: Registers,
}

impl Cpu {
    fn from_registers(registers: Registers) -> Self {
        Self { registers }
    }

    fn register(&self, index: Value) -> Result<&Value, Error> {
        self.registers
            .get(index as usize)
            .ok_or(Error::InvalidRegister)
    }

    fn register_mut(&mut self, index: Value) -> Result<&mut Value, Error> {
        self.registers
            .get_mut(index as usize)
            .ok_or(Error::InvalidRegister)
    }

    fn execute(&mut self, instruction: Instruction) -> Result<(), Error> {
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
            Gtir => (instruction.a > *self.register(instruction.b)?) as Value,
            Gtri => (*self.register(instruction.a)? > instruction.b) as Value,
            Gtrr => (*self.register(instruction.a)? > *self.register(instruction.b)?) as Value,
            Eqir => (instruction.a == *self.register(instruction.b)?) as Value,
            Eqri => (*self.register(instruction.a)? == instruction.b) as Value,
            Eqrr => (*self.register(instruction.a)? == *self.register(instruction.b)?) as Value,
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
            let mut cpu = Cpu::from_registers(self.before);
            cpu.execute(instruction).ok()?;
            let after: [Value; 4] = self.after;
            (cpu.registers == after).then(move || opcode)
        })
    }
}

impl FromStr for Sample {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        InputParser::parse_sample(s).map_err(Into::into)
    }
}

fn discover_opcodes_map(samples: &[Sample]) -> Result<HashMap<Value, Opcode>, Error> {
    let mut unknown_opcodes: HashSet<_> = Opcode::into_enum_iter().collect();
    let mut opcodes_map = HashMap::new();

    loop {
        let n_known = opcodes_map.len();
        for sample in samples {
            // if we've already figured this one out, move on
            if opcodes_map.contains_key(&sample.unknown_instruction.opcode) {
                continue;
            }

            let potential_opcodes: Vec<_> = sample
                .behaves_like()
                .filter(|opcode| unknown_opcodes.contains(opcode))
                .take(2)
                .collect();

            debug_assert_ne!(
                potential_opcodes.len(),
                0,
                "all samples must map to at least one opcode"
            );
            if let [opcode] = potential_opcodes.as_slice() {
                unknown_opcodes.remove(opcode);
                opcodes_map.insert(sample.unknown_instruction.opcode, *opcode);
            }
        }

        if unknown_opcodes.is_empty() {
            return Ok(opcodes_map);
        }
        if n_known == opcodes_map.len() {
            dbg!(&opcodes_map);
            // we haven't learned anything this iteration
            return Err(Error::NoSolution);
        }
    }
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

pub fn part2(input: &Path) -> Result<(), Error> {
    let input = InputParser::parse_file(input)?;
    let opcodes_map = discover_opcodes_map(&input.samples)?;
    let instructions = input
        .example_program
        .into_iter()
        .map(|unknown_instruction| unknown_instruction.assume_with(&opcodes_map));

    // no need for an instruction pointer or internal instructions because this CPU has no jumps
    let mut cpu = Cpu::default();
    for instruction in instructions {
        cpu.execute(instruction)?;
    }

    println!("value in register 0: {}", cpu.registers[0]);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("parse error")]
    Parse(#[from] pest_consume::Error<input_parser::Rule>),
    #[error("No solution found")]
    NoSolution,
    #[error("requested a register which does not exist")]
    InvalidRegister,
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
