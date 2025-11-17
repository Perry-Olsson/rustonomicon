use std::ptr::NonNull;

use crate::list::RawList;

pub(super) struct RawValIter<T> {
    pub(super) front: *const T,
    pub(super) back: *const T
}

impl <T> RawValIter<T> {
    pub(super) unsafe fn new(slice: &[T]) -> Self {
        RawValIter { 
            front: slice.as_ptr(),
            back: if std::mem::size_of::<T>() == 0 {
                ((slice.as_ptr() as usize) + slice.len()) as *const _
            } else if slice.len() == 0 {
                slice.as_ptr()
            } else {
                unsafe {
                    slice.as_ptr().add(slice.len())
                }
            }
        }
    }

    pub(super) fn next(&mut self) -> Option<T> {
        if self.front == self.back {
            None
        } else {
            unsafe {
                if std::mem::size_of::<T>() == 0 {
                    self.front = (self.front as usize + 1) as *const _;
                    Some(std::ptr::read(NonNull::dangling().as_ptr()))
                } else {
                    let val = std::ptr::read(self.front);
                    self.front = self.front.offset(1);
                    Some(val)
                }
            }
        }
    }

    pub(super) fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = std::mem::size_of::<T>();
        let len = (self.back as usize - self.front as usize)
                  / if elem_size == 0 { 1 } else { elem_size };
        (len, Some(len))
    }

    pub(super) fn next_back(&mut self) -> Option<T> {
        if self.front == self.back {
            None
        } else {
            unsafe {
                if std::mem::size_of::<T>() == 0 {
                    self.back = (self.back as usize - 1) as *const _;
                    Some(std::ptr::read(NonNull::dangling().as_ptr()))
                } else {
                    self.back = self.back.offset(-1);
                    Some(std::ptr::read(self.back))
                }
            }
        }
    }
}

pub struct IntoIter<T> {
    pub(super) _buf: RawList<T>,
    pub(super) iter: RawValIter<T>
}

impl <T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

impl <T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl <T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}
