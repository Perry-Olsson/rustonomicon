use std::marker::PhantomData;

use crate::list::{List, RawValIter};

pub struct Drain<'a, T: 'a> {
    pub(super) list: PhantomData<&'a mut List<T>>,
    pub(super) iter: RawValIter<T>
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

