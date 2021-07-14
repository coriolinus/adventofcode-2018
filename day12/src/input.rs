//! This module handles parsing the input.
//
// It's reliant on the `pest` parser generator plus the `pest_consume` helpers;
// this is strictly more powerful than LALRPOP, as it can handle newline limits
// much better, but it's still a little cumbersome. In particular, I wish that
// it was a bit more straightforward to define the transforms from nodes into
// real types.
//
// Still, this is a ton better than just using `pest` on its own, and `pest`
// itself is both easier to use and somewhat more powerful than lalrpop.
// This all counts as growing pains; I suspect that for non-trivial parsing
// in the future, I'll be reaching for this solution again.

use crate::{encode_as_u8::EncodeAsU8, Error, Rules};
use bitvec::vec::BitVec;
use pest_consume::{match_nodes, Parser};
use std::path::Path;

type Node<'i> = pest_consume::Node<'i, Rule, ()>;
type ParseResult<T> = Result<T, pest_consume::Error<Rule>>;

#[derive(Parser)]
#[grammar = "parser.pest"]
pub struct InputParser;

#[pest_consume::parser]
impl InputParser {
    fn EOI(_input: Node) -> ParseResult<()> {
        Ok(())
    }

    fn pot(input: Node) -> ParseResult<bool> {
        match input.as_str() {
            "." => Ok(false),
            "#" => Ok(true),
            _ => Err(input.error("expected '#' or '.'")),
        }
    }

    fn state(input: Node) -> ParseResult<BitVec> {
        Ok(match_nodes!(input.into_children();
            [pot(pots)..] => pots.collect(),
        ))
    }

    fn rule(input: Node) -> ParseResult<([bool; 5], bool)> {
        Ok(match_nodes!(input.into_children();
            [pot(p0), pot(p1), pot(p2), pot(p3), pot(p4), pot(p5)] => {
                ([p0, p1, p2, p3, p4], p5)
            }
        ))
    }

    fn rules(input: Node) -> ParseResult<Rules> {
        Ok(match_nodes!(input.into_children();
            [rule(rules)..] => {
                let mut rules_list = Rules::default();

                for (idx, val) in rules {
                    rules_list[idx.as_u8() as usize] = val;
                }

                rules_list
            },
        ))
    }

    fn file(input: Node) -> ParseResult<Input> {
        Ok(match_nodes!(input.into_children();
            [state(initial), rules(rules), EOI(_eoi)] => Input{ initial, rules },
        ))
    }
}

/// Representation of the day's input.
#[derive(Debug)]
pub struct Input {
    pub initial: BitVec,
    pub rules: Rules,
}

impl Input {
    pub fn load_file(path: &Path) -> Result<Self, Error> {
        let input_data = std::fs::read_to_string(path)?;
        Self::new(&input_data)
    }

    pub fn new(input_data: &str) -> Result<Self, Error> {
        let inputs = InputParser::parse(Rule::file, &input_data)?;
        let input = inputs.single()?;
        InputParser::file(input).map_err(Into::into)
    }
}
