//! This module handles parsing the input.
//
// It's reliant on the `pest` parser generator, which I have mixed feelings about.
// On the one hand, it's more powerful than LALRPOP; it Just Works on the input
// for this day, which LALRPOP didn't even with some research and tweaking.
//
// On the other hand, it doesn't provide a very convenient interface; having parsed
// the input, I now have to walk the parse tree manually and produce my desired types
// by hand. That doesn't feel great, particularly when combined with this bit of the
// documentation:
//
// > You _should_ rely on the maning of your grammar for properties such as "contains
// > _n_ sub-rules", "is safe to `parse` to `f32`", and "never fails to match". Idiomatic
// > `pest` code uses `unwrap` and `unreachable!`.
//
// That feels somewhat brittle; it's safe as long as all changes to `parser.pest`
// are always synchronized with changes here.

use crate::{encode_as_u8::EncodeAsU8, Error, Rules};
use bitvec::vec::BitVec;
use std::path::Path;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser.pest"]
pub struct InputParser;

/// Representation of the day's input.
#[derive(Debug)]
pub struct Input {
    pub initial: BitVec,
    pub rules: Rules,
}

impl Input {
    pub fn new(path: &Path) -> Result<Self, Error> {
        let input_data = std::fs::read_to_string(path)?;
        let mut file_pairs = InputParser::parse(Rule::file, &input_data)?
            .next()
            .unwrap()
            .into_inner();
        let initial: BitVec = file_pairs
            .next()
            .unwrap()
            .into_inner()
            .map(|rule| match rule.as_str() {
                "." => false,
                "#" => true,
                _ => unreachable!(),
            })
            .collect();

        let mut rules = Rules::default();
        for rule_def in file_pairs.next().unwrap().into_inner() {
            let mut bits = [false; 5];
            let mut bit_rule_iter = rule_def.into_inner();
            for (idx, bit_rule) in bit_rule_iter.by_ref().take(5).enumerate() {
                if bit_rule.as_str() == "#" {
                    bits[idx] = true;
                }
            }
            if bit_rule_iter.next().unwrap().as_str() == "#" {
                rules[bits.as_u8() as usize] = true;
            }
        }

        Ok(Self { initial, rules })
    }
}
