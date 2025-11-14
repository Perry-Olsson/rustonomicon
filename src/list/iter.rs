use crate::list::RawList;

pub(super) struct RawValIter<T> {
    pub(super) front: *const T,
    pub(super) back: *const T
}

impl <T> RawValIter<T> {
    pub(super) unsafe fn new(slice: &[T]) -> Self {
        RawValIter { 
            front: slice.as_ptr(),
            back: if slice.len() == 0 {
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
                let val = std::ptr::read(self.front);
                self.front = self.front.offset(1);
                return Some(val);
            }
        }
    }

    pub(super) fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.back as usize - self.front as usize)
                  / std::mem::size_of::<T>();
        (len, Some(len))
    }

    pub(super) fn next_back(&mut self) -> Option<T> {
        if self.front == self.back {
            None
        } else {
            unsafe {
                self.back = self.back.offset(-1);
                Some(std::ptr::read(self.back))
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
