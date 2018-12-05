use intrusive_collections::{intrusive_adapter, LinkedList, LinkedListLink};
use std::rc::Rc;

#[derive(Default)]
pub struct Node {
    pub value: char,
    link: LinkedListLink,
}

impl Node {
    fn new(c: char) -> Rc<Node> {
        Rc::new(Node {
            value: c,
            ..Node::default()
        })
    }
}

intrusive_adapter!(pub NodeAdapter = Rc<Node>: Node { link: LinkedListLink });

pub type Polymer = LinkedList<NodeAdapter>;

pub fn new(s: &str) -> Polymer {
    let mut polymer = Polymer::new(NodeAdapter::new());

    for c in s.chars() {
        polymer.push_back(Node::new(c));
    }

    polymer
}

pub fn to_string(p: &Polymer) -> String {
    let mut cursor = p.cursor();
    let mut out = String::new();

    loop {
        cursor.move_next();
        if let Some(node) = cursor.get() {
            out.push(node.value);
        } else {
            break;
        }
    }

    out
}
