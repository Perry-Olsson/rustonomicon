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
            std::ptr::write(self.ptr().add(self.back()), val)
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
        self.front = self.wrap(self.front);
        self.len -= 1;
        Some(val)
    }

    pub fn requeue(&mut self, val: T) {
        if self.is_full() {
            self.grow()
        }
        self.front = self.wrap_sub(self.front);
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

    pub fn size(&self) -> usize {
        self.len
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
                self.front = (self.front + 1) % old_cap
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
        (idx + 1) % self.cap()
    }

    fn wrap_sub(&self, idx: usize) -> usize {
        if idx == 0 {
            return self.cap() - 1;
        }
        idx - 1
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

    fn back(&self) -> usize {
        (self.front + self.len) % self.cap()
    }
}

impl <T> Drop for Queue<T> {
    fn drop(&mut self) {
        while let Some(_) = self.dequeue() { }
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
    extern crate stats_alloc;

    use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
    use std::alloc::System;

    #[global_allocator]
    static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

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
    fn ensure_heap_allocated_queue_items_are_dropped_if_left_in_queue() {
        let reg = Region::new(&GLOBAL);
        {
            let mut q: Queue<String> = nq();
            q.enqueue(String::from("hello"));
            q.enqueue(String::from("world"));
            q.dequeue();
        }
        let change = reg.change();
        assert_eq!(change.allocations, change.deallocations);
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

    #[test]
    fn test_enqueue_with_wrap_around() {
        let mut q = nq();
        q.enqueue(1);
        q.enqueue(2);
        q.enqueue(3);
        q.enqueue(4); // [f:1, 2, 3, b:4]
        assert_eq!(q.peek(), Some(&1), "Peek should return front element");
        q.dequeue();
        q.dequeue(); // [junk, junk, f:3, b:4]
        assert_eq!(q.size(), 2, "Queue should have 2 elements");
        assert_eq!(q.peek(), Some(&3), "Peek should return front element");
        q.enqueue(5); // [b:5, junk, f:3, 4]
        assert_eq!(q.cap(), 4, "Capacity should be 4");
        assert_eq!(q.size(), 3, "Queue should have 2 elements");
        assert_eq!(q.peek(), Some(&3), "Peek should return front element");
        q.enqueue(6); // [5, b:6, f:3, 4]
        assert_eq!(q.peek(), Some(&3), "Peek should return front element");
        q.enqueue(7); // [f:3, 4, 5, 6, b:7, junk, junk, junk]
        assert_eq!(q.cap(), 8, "Capacity should be 8");
        assert_eq!(q.size(), 5, "Queue should have 5 elements");
        assert_eq!(q.peek(), Some(&3), "Peek should return front element");
        assert_eq!(q.front, 0, "With current implementation front of queue should be 0 after resize");
    }

    #[test]
    fn dequeue_wraps_around() {
        let mut q = nq();
        q.enqueue(1);
        q.enqueue(2);
        q.enqueue(3);
        q.enqueue(4); // [f:1, 2, 3, b:4]
        q.dequeue();
        q.dequeue(); // [junk, junk, f:3, b:4]
        q.enqueue(5); // [b:5, junk, f:3, 4]
        q.enqueue(6); // [5, b:6, f:3, 4]
                      
        assert_eq!(q.dequeue(), Some(3));
        assert_eq!(q.size(), 3);
        assert_eq!(q.cap(), 4);

        assert_eq!(q.dequeue(), Some(4));
        assert_eq!(q.size(), 2, "Queue size should be 2");
        assert_eq!(q.cap(), 4, "Queue capacity mismatch");
        assert_eq!(q.front, 0, "Front of queue should be at index 0");

        assert_eq!(q.dequeue(), Some(5));
        assert_eq!(q.size(), 1, "Queue size mismatch");
        assert_eq!(q.cap(), 4, "Queue capacity should be 4");
        assert_eq!(q.front, 1, "Queue front mismatch. Expected: {}, Actual: {}", 0, q.front);
    }

    #[test]
    fn dequeue_to_empty_and_enqueue() {
        let mut q = nq();
        q.enqueue(1);
        q.enqueue(2);
        q.enqueue(3);
        q.enqueue(4); // [f:1, 2, 3, b:4]
        q.dequeue();
        q.dequeue(); // [junk, junk, f:3, b:4]
        q.enqueue(5); // [b:5, junk, f:3, 4]
        q.enqueue(6); // [5, b:6, f:3, 4]
        q.dequeue();
        q.dequeue();
        q.dequeue();
        q.dequeue();
        assert_eq!(None, q.dequeue());
        assert_eq!(0, q.size());
        assert_eq!(4, q.cap());
        assert_eq!(2, q.front);
        q.enqueue(7);
        assert_eq!(1, q.size());
        assert_eq!(4, q.cap());
        assert_eq!(2, q.front);
        assert_eq!(Some(&7), q.peek());
    }

    #[test]
    fn requeue_wraps_around_and_then_grows() {
        let mut q = nq();
        q.enqueue(1);
        q.enqueue(2); // [f:1, b:2, junk, junk]
        q.requeue(3);
        q.requeue(4); // [1, b:2, f:4, 3]
        assert_eq!(4, q.size());
        assert_eq!(4, q.cap());
        assert_eq!(2, q.front);
        assert_eq!(Some(&4), q.peek());
        q.requeue(5); // [4, 3, 1, B:2, junk, junk, junk, f:5]
        assert_eq!(5, q.size());
        assert_eq!(8, q.cap());
        assert_eq!(7, q.front);
        assert_eq!(Some(5), q.dequeue());
        assert_eq!(Some(4), q.dequeue());
        assert_eq!(Some(3), q.dequeue());
        assert_eq!(Some(1), q.dequeue());
        assert_eq!(Some(2), q.dequeue());
    }
}
