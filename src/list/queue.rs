use std::{alloc::Layout, fmt::{Display, Write}, ptr::NonNull};

use crate::list::RawList;

pub struct Queue<T> {
    buf: RawList<T>,
    len: usize,
    front: usize,
}

impl <T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            buf: RawList::new(),
            len: 0,
            front: 0,
        }
    }

    pub fn enqueue(&mut self, val: T) {
        if self.is_full() {
            self.grow()
        }

        unsafe {
            let idx = if self.len == 0 { 0 } else { self.len };
            std::ptr::write(self.ptr().add(idx), val)
        }

        self.len += 1;
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let val = unsafe {
            std::ptr::read(self.ptr().add(self.front))
        };
        self.front += 1;
        self.len -= 1;
        Some(val)
    }

    pub fn requeue(&mut self, val: T) {
        self.front -= 1;
        unsafe {
            std::ptr::write(self.ptr().add(self.front), val);
        }
        self.len += 1;
    }

    pub fn peek(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                let val = self.ptr().add(self.front);
                Some(&*val)
            }
        }
    }

    fn grow(&mut self) {
        assert!(std::mem::size_of::<T>() != 0, "capacity overflow");

        let (new_cap, new_layout) = if self.cap() == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = 2 * self.cap();
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_ptr = unsafe {
            std::alloc::alloc(new_layout)
        };
        let old_cap = self.cap();
        let old_ptr = self.ptr();
        self.buf.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => std::alloc::handle_alloc_error(new_layout),
        };
        self.buf.cap = new_cap;

        // TODO copy all values from previous buffer from front -> back with wrap
        for i in 0..self.len {
            unsafe {
                std::ptr::write(
                    self.buf.ptr.as_ptr().add(i),
                    std::ptr::read(old_ptr.add(self.front))
                );
                self.front = self.wrap(self.front);
            }
        }

        let old_layout = Layout::array::<T>(old_cap).unwrap();
        if old_cap != 0 {
            unsafe {
                std::alloc::dealloc(old_ptr as *mut u8, old_layout);
            }
        }
        self.front = 0;
    }

    fn wrap(&self, idx: usize) -> usize {
        (idx + 1) % self.len
    }

    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    fn cap(&self) -> usize {
        self.buf.cap
    }

    fn is_full(&self) -> bool {
        self.len == self.cap()
    }

    fn wrap_idx(&self, idx: usize) -> usize {
        (idx + 1) % self.cap()
    }
}

impl <T: Display> Display for Queue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        for i in self.front..self.len {
            let val = unsafe {
                std::ptr::read(self.buf.ptr.as_ptr().add(i))
            };
            f.write_str(&val.to_string()[..])?;
            if i != self.len - 1 {
                f.write_char(',')?;
            }
        }
        f.write_char(']')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a new queue
    fn nq<T>() -> Queue<T> {
        Queue::new()
    }

    #[test]
    fn test_new_queue_is_empty() {
        let q: Queue<i32> = nq();
        assert!(q.peek().is_none(), "New queue should be empty");
    }

    #[test]
    fn test_enqueue_single_element() {
        let mut q = nq();
        q.enqueue(42);
        assert_eq!(q.peek(), Some(&42), "Peek should return the enqueued element");
    }

    #[test]
    fn test_enqueue_dequeue_single_element() {
        let mut q = nq();
        q.enqueue(42);
        assert_eq!(q.dequeue(), Some(42), "Dequeue should return the enqueued element");
        assert!(q.peek().is_none(), "Queue should be empty after dequeue");
    }

    #[test]
    fn test_enqueue_multiple_elements() {
        let mut q = nq();
        q.enqueue(1);
        q.enqueue(2);
        q.enqueue(3);
        assert_eq!(q.peek(), Some(&1), "Peek should return first element");
    }

    #[test]
    fn test_dequeue_multiple_elements() {
        let mut q = nq();
        q.enqueue(1);
        q.enqueue(2);
        q.enqueue(3);
        assert_eq!(q.dequeue(), Some(1), "First dequeue should return 1");
        assert_eq!(q.dequeue(), Some(2), "Second dequeue should return 2");
        assert_eq!(q.dequeue(), Some(3), "Third dequeue should return 3");
        assert_eq!(q.dequeue(), None, "Dequeue on empty queue should return None");
    }

    #[test]
    fn test_requeue_single_element() {
        let mut q = nq();
        q.enqueue(1);
        q.dequeue();
        q.requeue(2);
        assert_eq!(q.peek(), Some(&2), "Requeue should place element at front");
        assert_eq!(q.dequeue(), Some(2), "Dequeue should return requeued element");
    }

    #[test]
    fn test_requeue_after_enqueue() {
        let mut q = nq();
        q.enqueue(1);
        q.enqueue(2);
        q.requeue(0);
        assert_eq!(q.peek(), Some(&0), "Requeue should place element at front");
        assert_eq!(q.dequeue(), Some(0), "First dequeue should return requeued element");
        assert_eq!(q.dequeue(), Some(1), "Second dequeue should return original first element");
    }

    #[test]
    fn test_peek_empty_queue() {
        let q: Queue<i32> = nq();
        assert_eq!(q.peek(), None, "Peek on empty queue should return None");
    }

    #[test]
    fn test_dequeue_empty_queue() {
        let mut q: Queue<i32> = nq();
        assert_eq!(q.dequeue(), None, "Dequeue on empty queue should return None");
    }

    #[test]
    fn test_queue_with_strings() {
        let mut q: Queue<String> = nq();
        q.enqueue(String::from("hello"));
        q.enqueue(String::from("world"));
        assert_eq!(q.peek(), Some(&String::from("hello")), "Peek should return first string");
        assert_eq!(q.dequeue(), Some(String::from("hello")), "Dequeue should return first string");
        assert_eq!(q.peek(), Some(&String::from("world")), "Peek should return second string");
    }

    #[test]
    fn test_multiple_operations() {
        let mut q = nq();
        q.enqueue(1);
        q.enqueue(2);
        assert_eq!(q.dequeue(), Some(1), "First dequeue should return 1");
        q.requeue(0);
        q.enqueue(3);
        assert_eq!(q.peek(), Some(&0), "Peek should return requeued element");
        assert_eq!(q.dequeue(), Some(0), "Dequeue should return requeued element");
        assert_eq!(q.dequeue(), Some(2), "Next dequeue should return 2");
        assert_eq!(q.dequeue(), Some(3), "Next dequeue should return 3");
        assert_eq!(q.dequeue(), None, "Queue should be empty");
    }

    #[test]
    fn test_requeue_empty_queue() {
        let mut q = nq();
        q.requeue(42);
        assert_eq!(q.peek(), Some(&42), "Requeue on empty queue should work");
        assert_eq!(q.dequeue(), Some(42), "Dequeue should return requeued element");
        assert_eq!(q.dequeue(), None, "Queue should be empty after dequeue");
    }
}
