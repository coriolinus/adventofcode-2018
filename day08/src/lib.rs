pub fn parse_input(s: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    s.split_whitespace().map(|n| n.parse()).collect()
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Node<'i> {
    children: Vec<Box<Node<'i>>>,
    metadata: &'i [u8],
}

impl<'i> Node<'i> {
    pub fn try_parse(mut input: &'i [u8]) -> Option<(Node, &[u8])> {
        if input.len() < 2 {
            return None;
        }

        let nchildren = input[0] as usize;
        let nmetadata = input[1] as usize;

        input = &input[2..];

        let mut out = Node::default();

        // parse child nodes
        for _ in 0..nchildren {
            let tpr = Node::try_parse(input)?;
            out.children.push(Box::new(tpr.0));
            input = tpr.1;
        }

        // store node metadata and remove it from input
        out.metadata = &input[..nmetadata];
        input = &input[nmetadata..];

        Some((out, input))
    }

    pub fn sum_metadata(&self) -> u32 {
        let mut out = self.metadata.iter().map(|&n| n as u32).sum();
        for child in self.children.iter() {
            out += child.sum_metadata();
        }
        out
    }

    fn child_value(&self, index: u8) -> u32 {
        let index = (index - 1) as usize; // metadata indices are 1-indices; our lists are 0-indices
        if index >= self.children.len() {
            0
        } else {
            self.children[index].value()
        }
    }

    pub fn value(&self) -> u32 {
        if self.children.is_empty() {
            self.metadata.iter().map(|&n| n as u32).sum()
        } else {
            self.metadata.iter().map(|&n| self.child_value(n)).sum()
        }
    }
}
