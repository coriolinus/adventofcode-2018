use std::path::Path;

fn parse_input(s: &str) -> Result<Vec<u8>, Error> {
    s.split_whitespace()
        .map(|n| n.parse().map_err(Into::into))
        .collect()
}

fn parse_input_file(input: &Path) -> Result<Vec<u8>, Error> {
    let s = std::fs::read_to_string(input)?;
    parse_input(&s)
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Node<'i> {
    children: Vec<Node<'i>>,
    metadata: &'i [u8],
}

impl<'i> Node<'i> {
    /// Try to parse this node from the given input.
    ///
    /// Returns the node and the unconsumed input.
    pub fn try_parse(mut input: &'i [u8]) -> Result<(Node<'_>, &[u8]), Error> {
        if input.len() < 2 {
            return Err(Error::NotEnoughInput);
        }

        let nchildren = input[0] as usize;
        let nmetadata = input[1] as usize;

        input = &input[2..];

        let mut out = Node::default();

        // parse child nodes
        for _ in 0..nchildren {
            let (child, remaining) = Node::try_parse(input)?;
            out.children.push(child);
            input = remaining;
        }

        // store node metadata and remove it from input
        // (can't use `.split_at(nmetadata)` because destructuring assignment _still_ isn't a thing,
        // (but it's getting closer now!))
        out.metadata = &input[..nmetadata];
        input = &input[nmetadata..];

        Ok((out, input))
    }

    pub fn iter(&'i self) -> Box<dyn 'i + Iterator<Item = &Node>> {
        Box::new(std::iter::once(self).chain(self.children.iter().flat_map(|child| child.iter())))
    }

    pub fn sum_metadata(&self) -> u32 {
        self.iter()
            .map(|node| node.metadata.iter().map(|&m| m as u32).sum::<u32>())
            .sum()
    }

    fn child_value(&self, index: u8) -> u32 {
        self.children
            // metadata indices are 1-indices; our lists are 0-indices
            .get(index as usize - 1)
            .map(|child| child.value())
            .unwrap_or_default()
    }

    pub fn value(&self) -> u32 {
        if self.children.is_empty() {
            self.sum_metadata()
        } else {
            self.metadata.iter().map(|&n| self.child_value(n)).sum()
        }
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let input = parse_input_file(input)?;
    let (node, remainder) = Node::try_parse(&input)?;
    if !remainder.is_empty() {
        eprintln!("found {} extra bytes in input", remainder.len());
    }

    println!("sum of metadata: {}", node.sum_metadata());
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let input = parse_input_file(input)?;
    let (node, _) = Node::try_parse(&input)?;

    println!("value of root: {}", node.value());
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("failed to parse integer")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Nodes require at least two digits to describe the quantity of children and metadata")]
    NotEnoughInput,
}
