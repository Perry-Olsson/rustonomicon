mod raw_list;
mod iter;

use raw_list::{RawList};
use iter::{IntoIter};
use std::{
    marker::PhantomData, mem::{self}, ops::{Deref, DerefMut}
};

use crate::list::iter::RawValIter;

pub struct List<T> {
    buf: RawList<T>,
    len: usize
}

unsafe impl<T: Send> Send for List<T> {}
unsafe impl<T: Sync> Sync for List<T> {}

impl <T> List<T> {
    pub fn new() -> List<T> {
        assert!(mem::size_of::<T>() != 0, "ZSTs can't be handled yet");
        List { 
            buf: RawList::new(),
            len: 0
        }
    }

    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    fn cap(&self) -> usize {
        self.buf.cap
    }

    pub fn push(&mut self, val: T) {
        if self.len == self.cap() {
            self.buf.grow()
        }

        unsafe {
            std::ptr::write(self.ptr().add(self.len), val)
        }

        self.len += 1;
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        if i >= self.len {
            None
        } else {
            unsafe {
                Some(&*self.ptr().add(i))
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(std::ptr::read(self.ptr().add(self.len)))
            }
        }
    }

    pub fn insert(&mut self, index: usize, val: T) {
        assert!(index <= self.len, "index out of bounds");
        if self.len == self.cap() { self.buf.grow() }

        unsafe {
            let insert_ptr = self.ptr().add(index);
            std::ptr::copy(
                insert_ptr,
                self.ptr().add(index + 1),
                self.len - index
            );
            std::ptr::write(insert_ptr, val);
        }
        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        unsafe {
            self.len -= 1;
            let removed = std::ptr::read(self.ptr().add(index));
            std::ptr::copy(
                self.ptr().add(index + 1),
                self.ptr().add(index),
                self.len - index
            );
            removed
        }
    }

    pub fn get_unchecked(&self, i: usize) -> &T {
        unsafe {
            &*self.ptr().add(i)
        }
    }

    pub fn drain(&mut self) -> Drain<T> {
        let iter = unsafe { RawValIter::new(&self) };

        self.len = 0;

        Drain {
            iter,
            list: PhantomData,
        }
    }
}

impl <T> Drop for List<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() { }
    }
}

impl <T> Deref for List<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.ptr(), self.len)
        }
    }
}

impl <T> DerefMut for List<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr(), self.len)
        }
    }
}

impl <T> IntoIterator for List<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            let iter = RawValIter::new(&self);

            let buf = std::ptr::read(&self.buf);
            mem::forget(self);

            IntoIter {
                iter,
                _buf: buf
            }
        }
    }
}

pub struct Drain<'a, T: 'a> {
    list: PhantomData<&'a mut List<T>>,
    iter: RawValIter<T>
}

impl <'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

impl <'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl <'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a new list
    fn nl<T>() -> List<T> {
        List::new()
    }

    #[test]
    fn test_new_list_is_empty() {
        let list: List<i32> = nl();
        assert_eq!(list.get(0), None, "New list should be empty");
    }

    #[test]
    fn test_push_single_element() {
        let mut list = nl();
        list.push(42);
        assert_eq!(list.get(0), Some(&42), "Get should return pushed element");
    }

    #[test]
    fn test_push_multiple_elements() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.push(3);
        assert_eq!(list.get(0), Some(&1), "Get index 0 should return 1");
        assert_eq!(list.get(1), Some(&2), "Get index 1 should return 2");
        assert_eq!(list.get(2), Some(&3), "Get index 2 should return 3");
    }

    #[test]
    fn test_pop_single_element() {
        let mut list = nl();
        list.push(42);
        assert_eq!(list.pop(), Some(42), "Pop should return the pushed element");
        assert_eq!(list.get(0), None, "List should be empty after pop");
    }

    #[test]
    fn test_pop_multiple_elements() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.push(3);
        assert_eq!(list.pop(), Some(3), "First pop should return last element");
        assert_eq!(list.pop(), Some(2), "Second pop should return second-to-last element");
        assert_eq!(list.pop(), Some(1), "Third pop should return first element");
        assert_eq!(list.pop(), None, "Pop on empty list should return None");
    }

    #[test]
    fn test_get_out_of_bounds() {
        let mut list = nl();
        list.push(1);
        assert_eq!(list.get(1), None, "Get beyond list length should return None");
    }

    #[test]
    fn test_index_operator() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        assert_eq!(list[0], 1, "Index 0 should return 1");
        assert_eq!(list[1], 2, "Index 1 should return 2");
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_index_operator_out_of_bounds() {
        let list: List<i32> = nl();
        let _ = list[0]; // Should panic since list is empty
    }

    #[test]
    fn test_get_unchecked() {
        let mut list = nl();
        list.push(42);
        assert_eq!(list.get_unchecked(0), &42, "get_unchecked should return element at index");
    }

    #[test]
    fn test_list_with_strings() {
        let mut list: List<String> = nl();
        list.push(String::from("hello"));
        list.push(String::from("world"));
        assert_eq!(list.get(0), Some(&String::from("hello")), "Get index 0 should return 'hello'");
        assert_eq!(list[1], String::from("world"), "Index 1 should return 'world'");
        assert_eq!(list.pop(), Some(String::from("world")), "Pop should return last string");
    }

    #[test]
    fn test_multiple_operations() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        assert_eq!(list.get(0), Some(&1), "Get index 0 should return 1");
        assert_eq!(list.pop(), Some(2), "Pop should return 2");
        list.push(3);
        assert_eq!(list[0], 1, "Index 0 should still return 1");
        assert_eq!(list.get(1), Some(&3), "Get index 1 should return 3");
        assert_eq!(list.pop(), Some(3), "Pop should return 3");
        assert_eq!(list.pop(), Some(1), "Pop should return 1");
        assert_eq!(list.pop(), None, "Pop on empty list should return None");
    }

    #[test]
    fn test_push_pop_empty() {
        let mut list = nl();
        assert_eq!(list.pop(), None, "Pop on empty list should return None");
        list.push(1);
        list.pop();
        assert_eq!(list.get(0), None, "List should be empty after push and pop");
    }

    #[test]
    fn test_insert_into_empty_list() {
        let mut list: List<i32> = nl();
        list.insert(0, 42);
        assert_eq!(list.get(0), Some(&42), "Insert into empty list should place element at index 0");
        assert_eq!(list.len, 1, "Length should be 1 after insert");
        assert_eq!(list.get(1), None, "Index 1 should be out of bounds");
    }

    #[test]
    fn test_insert_at_start() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.insert(0, 0);
        assert_eq!(list.get(0), Some(&0), "Insert at start should place element at index 0");
        assert_eq!(list.get(1), Some(&1), "Original first element should be shifted");
        assert_eq!(list.get(2), Some(&2), "Original second element should be shifted");
        assert_eq!(list.len, 3, "Length should be 3 after insert");
    }

    #[test]
    fn test_insert_at_end() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.insert(2, 3);
        assert_eq!(list.get(0), Some(&1), "First element should remain");
        assert_eq!(list.get(1), Some(&2), "Second element should remain");
        assert_eq!(list.get(2), Some(&3), "Insert at end should place element at index 2");
        assert_eq!(list.len, 3, "Length should be 3 after insert");
    }

    #[test]
    fn test_insert_in_middle() {
        let mut list = nl();
        list.push(1);
        list.push(3);
        list.insert(1, 2);
        assert_eq!(list.get(0), Some(&1), "First element should remain");
        assert_eq!(list.get(1), Some(&2), "Inserted element should be at index 1");
        assert_eq!(list.get(2), Some(&3), "Original second element should be shifted");
        assert_eq!(list.len, 3, "Length should be 3 after insert");
    }

    #[test]
    fn test_insert_with_strings() {
        let mut list: List<String> = nl();
        list.push(String::from("a"));
        list.push(String::from("c"));
        list.insert(1, String::from("b"));
        assert_eq!(list.get(0), Some(&String::from("a")), "First element should remain");
        assert_eq!(list.get(1), Some(&String::from("b")), "Inserted element should be at index 1");
        assert_eq!(list.get(2), Some(&String::from("c")), "Original second element should be shifted");
        assert_eq!(list.len, 3, "Length should be 3 after insert");
    }

    #[test]
    fn test_insert_triggers_growth() {
        let mut list = nl();
        // Fill to capacity (assuming initial capacity is 0, grows to 1, then 2, etc.)
        list.push(1); // cap = 1
        list.push(2); // cap = 2
        list.push(3); // cap = 4
        list.push(4);
        let initial_cap = list.cap();
        list.insert(1, 5); // Should trigger growth if cap is full
        assert_eq!(list.get(0), Some(&1), "First element should remain");
        assert_eq!(list.get(1), Some(&5), "Inserted element should be at index 1");
        assert_eq!(list.get(2), Some(&2), "Second element should be shifted");
        assert_eq!(list.get(3), Some(&3), "Third element should be shifted");
        assert_eq!(list.get(4), Some(&4), "Third element should be shifted");
        assert_eq!(list.len, 5, "Length should be 4 after insert");
        assert!(list.cap() > initial_cap, "Capacity should increase after insert");
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_insert_out_of_bounds() {
        let mut list = nl();
        list.push(1);
        list.insert(2, 2); // Should panic since index 2 > len
    }

    #[test]
    fn test_multiple_inserts() {
        let mut list = nl();
        list.push(2);
        list.push(4);
        list.insert(0, 1);
        list.insert(2, 3);
        list.insert(4, 5);
        assert_eq!(list.get(0), Some(&1), "First insert should be at index 0");
        assert_eq!(list.get(1), Some(&2), "Original first element shifted");
        assert_eq!(list.get(2), Some(&3), "Second insert should be at index 2");
        assert_eq!(list.get(3), Some(&4), "Original second element shifted");
        assert_eq!(list.get(4), Some(&5), "Third insert should be at index 4");
        assert_eq!(list.len, 5, "Length should be 5 after inserts");
    }

    #[test]
    fn test_insert_with_deref() {
        let mut list = nl();
        list.push(1);
        list.push(3);
        list.insert(1, 2);
        let slice: &[i32] = &list;
        assert_eq!(slice, &[1, 2, 3], "Deref should return correct slice after insert");
    }

    #[test]
    fn test_remove_from_single_element_list() {
        let mut list = nl();
        list.push(42);
        let removed = list.remove(0);
        assert_eq!(removed, 42, "Remove should return the removed element");
        assert_eq!(list.len, 0, "Length should be 0 after remove");
        assert_eq!(list.get(0), None, "List should be empty after remove");
    }

    #[test]
    fn test_remove_from_start() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.push(3);
        let removed = list.remove(0);
        assert_eq!(removed, 1, "Remove should return first element");
        assert_eq!(list.get(0), Some(&2), "Second element should shift to index 0");
        assert_eq!(list.get(1), Some(&3), "Third element should shift to index 1");
        assert_eq!(list.len, 2, "Length should be 2 after remove");
    }

    #[test]
    fn test_remove_from_end() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.push(3);
        let removed = list.remove(2);
        assert_eq!(removed, 3, "Remove should return last element");
        assert_eq!(list.get(0), Some(&1), "First element should remain");
        assert_eq!(list.get(1), Some(&2), "Second element should remain");
        assert_eq!(list.get(2), None, "Index 2 should be out of bounds");
        assert_eq!(list.len, 2, "Length should be 2 after remove");
    }

    #[test]
    fn test_remove_from_middle() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.push(3);
        let removed = list.remove(1);
        assert_eq!(removed, 2, "Remove should return middle element");
        assert_eq!(list.get(0), Some(&1), "First element should remain");
        assert_eq!(list.get(1), Some(&3), "Third element should shift to index 1");
        assert_eq!(list.get(2), None, "Index 2 should be out of bounds");
        assert_eq!(list.len, 2, "Length should be 2 after remove");
    }

    #[test]
    fn test_remove_with_strings() {
        let mut list: List<String> = nl();
        list.push(String::from("a"));
        list.push(String::from("b"));
        list.push(String::from("c"));
        let removed = list.remove(1);
        assert_eq!(removed, String::from("b"), "Remove should return middle string");
        assert_eq!(list.get(0), Some(&String::from("a")), "First string should remain");
        assert_eq!(list.get(1), Some(&String::from("c")), "Third string should shift to index 1");
        assert_eq!(list.len, 2, "Length should be 2 after remove");
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_remove_out_of_bounds() {
        let mut list = nl();
        list.push(1);
        list.remove(1); // Should panic since index 1 >= len
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_remove_from_empty_list() {
        let mut list: List<i32> = nl();
        list.remove(0); // Should panic since list is empty
    }

    #[test]
    fn test_multiple_removes() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.push(3);
        list.push(4);
        let removed1 = list.remove(0);
        let removed2 = list.remove(1);
        assert_eq!(removed1, 1, "First remove should return 1");
        assert_eq!(removed2, 3, "Second remove should return 3");
        assert_eq!(list.get(0), Some(&2), "Second element should be at index 0");
        assert_eq!(list.get(1), Some(&4), "Fourth element should be at index 1");
        assert_eq!(list.len, 2, "Length should be 2 after removes");
    }

    #[test]
    fn test_remove_with_deref() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.push(3);
        list.remove(1);
        let slice: &[i32] = &list;
        assert_eq!(slice, &[1, 3], "Deref should return correct slice after remove");
    }

    #[test]
    fn test_remove_preserves_capacity() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        let initial_cap = list.cap();
        list.remove(0);
        assert_eq!(list.cap(), initial_cap, "Capacity should not change after remove");
        assert_eq!(list.len, 1, "Length should be 1 after remove");
        assert_eq!(list.get(0), Some(&2), "Remaining element should be correct");
    }

    #[test]
    fn test_into_iter_empty_list() {
        let list: List<i32> = nl();
        let mut iter = list.into_iter();
        assert_eq!(iter.next(), None, "Empty iterator should yield None");
        assert_eq!(iter.next_back(), None, "Empty iterator should yield None for next_back");
        assert_eq!(iter.size_hint(), (0, Some(0)), "Size hint should be (0, Some(0)) for empty iterator");
    }

    #[test]
    fn test_into_iter_single_element() {
        let mut list = nl();
        list.push(42);
        let mut iter = list.into_iter();
        assert_eq!(iter.size_hint(), (1, Some(1)), "Size hint should be (1, Some(1))");
        assert_eq!(iter.next(), Some(42), "Iterator should yield single element");
        assert_eq!(iter.next(), None, "Iterator should be exhausted after one element");
        assert_eq!(iter.next_back(), None, "Exhausted iterator should yield None for next_back");
        assert_eq!(iter.size_hint(), (0, Some(0)), "Size hint should be (0, Some(0)) after exhaustion");
    }

    #[test]
    fn test_into_iter_multiple_elements_forward() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut iter = list.into_iter();
        assert_eq!(iter.size_hint(), (3, Some(3)), "Size hint should be (3, Some(3))");
        assert_eq!(iter.next(), Some(1), "First element should be 1");
        assert_eq!(iter.size_hint(), (2, Some(2)), "Size hint should update after next");
        assert_eq!(iter.next(), Some(2), "Second element should be 2");
        assert_eq!(iter.next(), Some(3), "Third element should be 3");
        assert_eq!(iter.next(), None, "Iterator should be exhausted");
        assert_eq!(iter.size_hint(), (0, Some(0)), "Size hint should be (0, Some(0)) after exhaustion");
    }

    #[test]
    fn test_into_iter_multiple_elements_backward() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut iter = list.into_iter();
        assert_eq!(iter.next_back(), Some(3), "Last element should be 3");
        assert_eq!(iter.size_hint(), (2, Some(2)), "Size hint should update after next_back");
        assert_eq!(iter.next_back(), Some(2), "Second-to-last element should be 2");
        assert_eq!(iter.next_back(), Some(1), "First element should be 1");
        assert_eq!(iter.next_back(), None, "Iterator should be exhausted");
        assert_eq!(iter.next(), None, "Exhausted iterator should yield None for next");
    }

    #[test]
    fn test_into_iter_mixed_forward_backward() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(1), "First element should be 1");
        assert_eq!(iter.next_back(), Some(3), "Last element should be 3");
        assert_eq!(iter.size_hint(), (1, Some(1)), "Size hint should be (1, Some(1))");
        assert_eq!(iter.next(), Some(2), "Middle element should be 2");
        assert_eq!(iter.next(), None, "Iterator should be exhausted");
        assert_eq!(iter.next_back(), None, "Exhausted iterator should yield None");
    }

    #[test]
    fn test_into_iter_with_strings() {
        let mut list: List<String> = nl();
        list.push(String::from("a"));
        list.push(String::from("b"));
        let mut iter = list.into_iter();
        assert_eq!(iter.size_hint(), (2, Some(2)), "Size hint should be (2, Some(2))");
        assert_eq!(iter.next(), Some(String::from("a")), "First element should be 'a'");
        assert_eq!(iter.next_back(), Some(String::from("b")), "Last element should be 'b'");
        assert_eq!(iter.next(), None, "Iterator should be exhausted");
    }

    #[test]
    fn test_into_iter_for_loop() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut result = Vec::new();
        for x in list {
            result.push(x);
        }
        assert_eq!(result, vec![1, 2, 3], "For loop should yield elements in order");
    }

    #[test]
    fn test_into_iter_collect() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        list.push(3);
        let result: Vec<i32> = list.into_iter().collect();
        assert_eq!(result, vec![1, 2, 3], "Collect should yield elements in order");
    }

    #[test]
    fn test_into_iter_drop_early() {
        let mut list = nl();
        list.push(1);
        list.push(2);
        {
            let mut iter = list.into_iter();
            assert_eq!(iter.next(), Some(1), "First element should be 1");
            // Drop iterator early (before consuming all elements)
        }
        // No assertion needed; test passes if no memory leaks or panics occur
    }

    #[test]
    fn test_into_iter_size_hint_edge_cases() {
        let mut list = nl();
        list.push(1);
        let mut iter = list.into_iter();
        assert_eq!(iter.size_hint(), (1, Some(1)), "Initial size hint should be (1, Some(1))");
        iter.next();
        assert_eq!(iter.size_hint(), (0, Some(0)), "Size hint should be (0, Some(0)) after consuming");
        iter.next_back();
        assert_eq!(iter.size_hint(), (0, Some(0)), "Size hint should remain (0, Some(0)) after exhaustion");
    }
}
