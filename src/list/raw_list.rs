use std::{alloc::{self, Layout}, ptr::NonNull};

pub(super) struct RawList<T> {
    pub(super) ptr: NonNull<T>,
    pub(super) cap: usize
}

unsafe impl<T: Send> Send for RawList<T> {}
unsafe impl<T: Sync> Sync for RawList<T> {}

impl <T> RawList<T> {
    pub fn new() -> RawList<T> {
        assert!(std::mem::size_of::<T>() != 0, "ZSTs can't be handled yet");
        RawList { 
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }

    pub(super) fn grow(&mut self) {
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
}

impl <T> Drop for RawList<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}
