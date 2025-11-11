use std::{
    alloc::{self, Layout},
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull
};

pub struct List<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize
}

unsafe impl<T: Send> Send for List<T> {}
unsafe impl<T: Sync> Sync for List<T> {}

impl <T> List<T> {
    pub fn new() -> List<T> {
        assert!(mem::size_of::<T>() != 0, "ZSTs can't be handled yet");
        List { 
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0
        }
    }

    fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = 2 * self.cap;
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }

    pub fn push(&mut self, val: T) {
        if self.len == self.cap {
            self.grow()
        }

        unsafe {
            std::ptr::write(self.ptr.as_ptr().add(self.len), val)
        }

        self.len += 1;
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        if i >= self.len {
            None
        } else {
            unsafe {
                Some(&*self.ptr.as_ptr().add(i))
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(std::ptr::read(self.ptr.as_ptr().add(self.len)))
            }
        }
    }

    pub fn get_unchecked(&self, i: usize) -> &T {
        unsafe {
            &*self.ptr.as_ptr().add(i)
        }
    }
}

impl <T> Drop for List<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }

        while let Some(_) = self.pop() { }
        let layout = Layout::array::<T>(self.cap).unwrap();
        unsafe {
            alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
        }
    }
}

impl <T> Deref for List<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }
}

impl <T> DerefMut for List<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
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
}
