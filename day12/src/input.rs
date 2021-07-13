use crate::{encode_as_u8::EncodeAsU8, Error, Rules};
use std::path::Path;

pub type Rule = ([bool; 5], bool);

/// Representation of the day's input.
pub struct Input {
    pub initial: Vec<bool>,
    pub rules: Vec<Rule>,
}

impl Input {
    pub fn new(path: &Path) -> Result<Self, Error> {
        let input_data = std::fs::read_to_string(path)?;
        crate::parser::InputParser::new()
            .parse(&input_data)
            .map_err(|err| {
                err.map_token(|token| token.to_string())
                    .map_error(|err| err.to_string())
                    .into()
            })
    }

    pub fn rules(&self) -> Rules {
        let mut rules = Rules::default();

        for (rule, sets) in self.rules.iter().copied() {
            rules[rule.as_u8() as usize] = sets;
        }

        rules
    }
}
