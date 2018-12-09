//! this implementation is heavily based on the reddit post
//! [here](https://www.reddit.com/r/rust/comments/7zsy72/writing_a_doubly_linked_list_in_rust_is_easy/),
//! which in turn references [this playground link](https://play.rust-lang.org/?gist=d65d605a48d38648737ad2ae38f46434&version=stable)

use slab::Slab;
use std::fmt;
use std::ops::{Index, IndexMut};

// A doubly linked list.
pub struct Circle<T> {
    // All nodes get stored into this slab. A slab is basically just a
    // `Vec<Option<T>>` in disguse. We use it as a simple node allocator.
    slab: Slab<Node<T>>,
    // The head of the doubly linked list.
    pub head: Pointer,
    // The tail of the doubly linked list.
    pub tail: Pointer,
    // number of nodes in this list
    size: usize,
}

// A node in a doubly-linked list.
pub struct Node<T> {
    // The value stored in this node.
    value: T,
    // The next node in the list.
    pub next: Pointer,
    // The previous node in the list.
    pub prev: Pointer,
}

// A `Pointer` is just an index that refers to a node in the slab.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Pointer(usize);

impl Pointer {
    // The null pointer is `!0`, which is the largest possible value of type
    // `usize`. There's no way we'll ever have a legitimate index that large.
    #[inline]
    fn null() -> Pointer {
        Pointer(!0)
    }

    // Returns `true` if this pointer is null.
    #[inline]
    pub fn is_null(&self) -> bool {
        *self == Pointer::null()
    }
}

// Just for convenience, so that we can type `self[i]` instead of `self.slab[i]`.
impl<T> Index<Pointer> for Circle<T> {
    type Output = Node<T>;

    fn index(&self, index: Pointer) -> &Node<T> {
        &self.slab[index.0]
    }
}

// Just for convenience, so that we can type `self[i]` instead of `self.slab[i]`.
impl<T> IndexMut<Pointer> for Circle<T> {
    fn index_mut(&mut self, index: Pointer) -> &mut Node<T> {
        &mut self.slab[index.0]
    }
}

impl<T> Circle<T> {
    // Returns a new doubly linked list.
    pub fn new() -> Circle<T> {
        Circle {
            slab: Slab::new(),
            head: Pointer::null(),
            tail: Pointer::null(),
            size: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Circle<T> {
        Circle {
            slab: Slab::with_capacity(capacity),
            ..Circle::new()
        }
    }

    // Inserts a new element at the back of the list.
    pub fn push_back(&mut self, t: T) -> Pointer {
        self.size += 1;

        let tail = self.tail;
        if tail.is_null() {
            let n = Pointer(self.slab.insert(Node {
                value: t,
                prev: Pointer::null(),
                next: Pointer::null(),
            }));
            self.head = n;
            self.tail = n;
            n
        } else {
            self.insert_after(tail, t)
        }
    }

    // Inserts a new element at the front of the list.
    pub fn push_front(&mut self, t: T) -> Pointer {
        self.size += 1;

        let head = self.head;
        if head.is_null() {
            self.push_back(t)
        } else {
            self.insert_before(head, t)
        }
    }

    // Inserts a new element after `node`.
    pub fn insert_after(&mut self, node: Pointer, t: T) -> Pointer {
        self.size += 1;

        let next = self[node].next;
        let n = Pointer(self.slab.insert(Node {
            value: t,
            prev: node,
            next: next,
        }));

        if next.is_null() {
            self.tail = n;
        } else {
            self[next].prev = n;
        }
        self[node].next = n;
        n
    }

    // Inserts a new element before `node`.
    pub fn insert_before(&mut self, node: Pointer, t: T) -> Pointer {
        self.size += 1;

        let prev = self[node].prev;
        let n = Pointer(self.slab.insert(Node {
            value: t,
            prev: prev,
            next: node,
        }));

        if prev.is_null() {
            self.head = n;
        } else {
            self[prev].next = n;
        }
        self[node].prev = n;
        n
    }

    // Removes `node` from the list and returns its value.
    pub fn remove(&mut self, node: Pointer) -> T {
        self.size -= 1;

        let prev = self[node].prev;
        let next = self[node].next;

        if prev.is_null() {
            self.head = next;
        } else {
            self[prev].next = next;
        }

        if next.is_null() {
            self.tail = prev;
        } else {
            self[next].prev = prev;
        }

        self.slab.remove(node.0).value
    }

    // return the number of nodes in this list
    pub fn len(&self) -> usize {
        self.size
    }
}

impl<T: fmt::Debug> fmt::Debug for Circle<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        let mut n = self.head;

        write!(f, "List(")?;
        while !n.is_null() {
            if !first {
                write!(f, ", ")?;
            }
            first = false;

            write!(f, "{:?}", self[n].value)?;
            n = self[n].next;
        }
        write!(f, ")")?;

        Ok(())
    }
}
