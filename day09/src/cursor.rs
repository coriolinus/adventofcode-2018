use crate::circle::{Circle, Pointer};

#[derive(Debug)]
pub struct Cursor<T> {
    circle: Circle<T>,
    pointer: Pointer,
}

impl<T> Cursor<T> {
    pub fn new(circle: Circle<T>) -> Cursor<T> {
        Cursor {
            pointer: circle.head,
            circle,
        }
    }

    pub fn into_circle(self) -> Circle<T> {
        self.circle
    }

    #[inline]
    pub fn step_right(&mut self) {
        self.pointer = self.circle[self.pointer].next;
        if self.pointer.is_null() {
            self.pointer = self.circle.head;
        }
    }

    #[inline]
    pub fn step_left(&mut self) {
        self.pointer = self.circle[self.pointer].prev;
        if self.pointer.is_null() {
            self.pointer = self.circle.tail;
        }
    }

    pub fn seek(&mut self, steps: isize) {
        match steps {
            0 => (),
            n if n > 0 => {
                let n = (n as usize) % self.circle.len();
                for _ in 0..n {
                    self.step_right();
                }
            }
            n if n < 0 => {
                let n = (-n as usize) % self.circle.len();
                for _ in 0..n {
                    self.step_left();
                }
            }
            _ => unreachable!(),
        }
    }

    /// insert the given value before the current value, updating the current
    /// value to the inserted
    pub fn insert(&mut self, t: T) {
        self.pointer = self.circle.insert_before(self.pointer, t);
    }

    /// remove the current value, updating the current value to the one to the
    /// right of the removed value
    pub fn remove(&mut self) -> T {
        let current = self.pointer.clone();
        self.step_right();
        self.circle.remove(current)
    }
}
