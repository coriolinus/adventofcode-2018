use aoclib::input::parse_newline_sep;
use enum_iterator::IntoEnumIterator;
use std::path::Path;

type Value = i32;

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
#[display("[{0}, {1}, {2}, {3}]")]
struct Registers(Value, Value, Value, Value);

impl From<Registers> for [Value; 4] {
    fn from(registers: Registers) -> Self {
        [registers.0, registers.1, registers.2, registers.3]
    }
}

struct Cpu {
    registers: [Value; 4],
}

impl Cpu {
    fn new(registers: [Value; 4]) -> Self {
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
#[display("Before: {before}\n{unknown_instruction}\nAfter:  {after}")]
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

#[derive(Debug, thiserror::Error)]
enum CpuError {
    #[error("requested a register which does not exist")]
    InvalidRegister,
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let samples_with_at_lest_three_possibilities = parse_newline_sep::<Sample>(input)?
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
