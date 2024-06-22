use std::{
    array::from_fn,
    mem::MaybeUninit,
    ops::{Index, IndexMut},
    usize,
};

#[derive(Debug)]
pub struct ArrDeque<T, const N: usize> {
    /// The backing array of this ArrDeque.
    items: [MaybeUninit<T>; N],
    /// The front index (inclusive).
    front: usize,
    /// The back index (inclusive).
    back: usize,
    /// Because front will equal back at both size 0
    /// and size items.len(), we must keep track of
    /// the len and not calculate it dynamically.
    len: usize,
}

impl<T, const N: usize> Clone for ArrDeque<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut iter_finished = false;
        let mut iter = self.iter();
        let new_arr = from_fn(|_| {
            if iter_finished {
                MaybeUninit::uninit()
            } else if let Some(next) = iter.next() {
                MaybeUninit::new(next.clone())
            } else {
                iter_finished = true;
                MaybeUninit::uninit()
            }
        });

        Self {
            items: new_arr,
            front: 0,
            back: self.len - 1,
            len: self.len,
        }
    }
}

impl<T, const N: usize> Default for ArrDeque<T, N>
where
    T: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> ArrDeque<T, N> {
    pub fn new() -> Self {
        Self {
            items: from_fn(|_| MaybeUninit::uninit()),
            front: 0,
            back: N - 1,
            len: 0,
        }
    }

    pub fn from_fn<F>(mut cb: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        Self {
            items: from_fn(|i| MaybeUninit::new(cb(i))),
            front: 0,
            back: N - 1,
            len: N,
        }
    }

    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[allow(unused)]
    pub fn is_full(&self) -> bool {
        self.len() == self.items.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        unsafe {
            self.drop_elements();
        }
        self.back = if self.front == 0 {
            N - 1
        } else {
            self.front - 1
        };
        self.len = 0;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }

        let result = std::mem::replace(&mut self.items[self.front], MaybeUninit::uninit());
        self.front += 1;
        if self.front == self.items.len() {
            self.front = 0;
        }
        self.len -= 1;
        return Some(unsafe { result.assume_init() });
    }

    pub fn peek_front(&self) -> Option<&T> {
        if self.len() == 0 {
            return None;
        }
        let maybe_uninit = &self.items[self.front];
        let value = unsafe { maybe_uninit.assume_init_ref() };
        return Some(value);
    }

    pub fn push_front(&mut self, item: T) -> Result<(), ()> {
        debug_assert!(self.len() <= self.items.len(), "{}", DATA_INTEGRITY_ERR_MSG);
        if self.len() == self.items.len() {
            return Err(());
        }

        if self.front == 0 {
            self.front = self.items.len() - 1;
        } else {
            self.front -= 1;
        }
        self.len += 1;
        self.items[self.front] = MaybeUninit::new(item);

        return Ok(());
    }

    #[allow(unused)]
    pub fn pop_back(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }

        let maybe_uninit = std::mem::replace(&mut self.items[self.back], MaybeUninit::uninit());
        if self.back == 0 {
            self.back = self.items.len();
        }
        self.len -= 1;
        self.back -= 1;
        let value = unsafe { maybe_uninit.assume_init() };
        return Some(value);
    }

    #[allow(unused)]
    pub fn peek_back(&self) -> Option<&T> {
        if self.len() == 0 {
            return None;
        }
        let maybe_uninit = &self.items[self.back];
        let value = unsafe { maybe_uninit.assume_init_ref() };
        return Some(value);
    }

    pub fn push_back(&mut self, item: T) -> Result<(), ()> {
        debug_assert!(self.len() <= self.items.len(), "{}", DATA_INTEGRITY_ERR_MSG);
        if self.len() == self.items.len() {
            return Err(());
        }

        self.len += 1;
        self.back = (self.back + 1) % (self.items.len());
        self.items[self.back] = MaybeUninit::new(item);

        Ok(())
    }

    pub fn iter(&self) -> Iter<T, N> {
        Iter {
            deque: self,
            indexes: self.active_indexes(),
        }
    }

    #[allow(unused)]
    pub fn iter_mut(&mut self) -> IterMut<'_, T, N> {
        let indexes = self.active_indexes();
        IterMut {
            deque: self,
            indexes: indexes,
        }
    }

    unsafe fn drop_elements(&mut self) {
        for index in self.active_indexes() {
            unsafe {
                self.items[index].assume_init_drop();
            };
        }
    }

    fn active_indexes(&self) -> IndexIterator {
        IndexIterator {
            deque_front: self.front,
            deque_back: self.back,
            deque_capacity: self.items.len(),
            deque_len: self.len(),
            current: if self.front == 0 {
                self.items.len() - 1
            } else {
                self.front - 1
            },
            num_iterations: 0,
        }
    }
}

#[derive(Debug)]
struct IndexIterator {
    deque_front: usize,
    deque_back: usize,
    deque_capacity: usize,
    deque_len: usize,
    current: usize,
    num_iterations: usize,
}

impl Iterator for IndexIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num_iterations >= self.deque_len {
            return None;
        }
        self.num_iterations += 1;

        self.current = (self.current + 1) % self.deque_capacity;

        if self.deque_front <= self.deque_back {
            if self.current >= self.deque_front && self.current <= self.deque_back {
                return Some(self.current);
            } else {
                return None;
            }
        } else if self.current <= self.deque_back
            || (self.current >= self.deque_front && self.current < self.deque_capacity)
        {
            return Some(self.current);
        } else {
            return None;
        }
    }
}

impl<T, const N: usize> Drop for ArrDeque<T, N> {
    fn drop(&mut self) {
        unsafe {
            self.drop_elements();
        }
    }
}

const DATA_INTEGRITY_ERR_MSG: &'static str = "DATA INTEGRITY VIOLATION";
impl<T, const N: usize> Index<usize> for ArrDeque<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let maybe_uninit = &self.items[(self.front + index) % self.items.len()];
        unsafe { maybe_uninit.assume_init_ref() }
    }
}

impl<T, const N: usize> IndexMut<usize> for ArrDeque<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let maybe_uninit = &mut self.items[(self.front + index) % self.items.len()];
        unsafe { maybe_uninit.assume_init_mut() }
    }
}

impl<T, const N: usize> IntoIterator for ArrDeque<T, N> {
    type Item = T;

    type IntoIter = IntoIter<T, N>;

    fn into_iter(mut self) -> Self::IntoIter {
        let indexes = self.active_indexes();
        // We're handling control of the objects over to the caller, so we need
        // to not call the destructor on them.
        self.len = 0;
        self.back = if self.front == 0 {
            self.items.len()
        } else {
            self.front - 1
        };
        IntoIter {
            deque: self,
            indexes,
        }
    }
}

#[derive(Debug)]
pub struct IntoIter<T, const N: usize> {
    deque: ArrDeque<T, N>,
    indexes: IndexIterator,
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.indexes.next() {
            let maybe_uninit =
                std::mem::replace(&mut self.deque.items[index], MaybeUninit::uninit());
            let value = unsafe { maybe_uninit.assume_init() };
            return Some(value);
        }
        return None;
    }
}

#[derive(Debug)]
pub struct IterMut<'deque, T, const N: usize> {
    deque: &'deque mut ArrDeque<T, N>,
    indexes: IndexIterator,
}

impl<'deque, T, const N: usize> Iterator for IterMut<'deque, T, N> {
    type Item = &'deque mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.indexes.next() {
            let maybe_uninit = &mut self.deque.items[index] as *mut MaybeUninit<T>;
            return Some(unsafe { (*maybe_uninit).assume_init_mut() });
        }

        return None;
    }
}

#[derive(Debug)]
pub struct Iter<'deque, T, const N: usize> {
    deque: &'deque ArrDeque<T, N>,
    indexes: IndexIterator,
}

impl<'deque, T, const N: usize> Iterator for Iter<'deque, T, N> {
    type Item = &'deque T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.indexes.next() {
            let maybe_uninit = &self.deque.items[index];
            let value = unsafe { maybe_uninit.assume_init_ref() };
            return Some(value);
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::ArrDeque;

    #[test]
    fn push_back_works() {
        let mut deque: ArrDeque<i32, 3> = ArrDeque::default();
        assert!(deque.push_back(1).is_ok());
        assert!(deque.len() == 1, "{}", deque.len());

        assert!(deque.push_back(2).is_ok());
        assert!(deque.len() == 2, "{}", deque.len());

        assert!(deque.push_back(3).is_ok());
        assert!(deque.len() == 3, "{}", deque.len());

        assert!(deque.push_back(4).is_err());
        assert!(deque.len() == 3, "{}", deque.len());
    }

    #[test]
    fn push_front_works() {
        let mut deque: ArrDeque<i32, 3> = ArrDeque::default();
        assert!(deque.push_front(1).is_ok());
        assert!(deque.len() == 1, "{}", deque.len());

        assert!(deque.push_front(2).is_ok());
        assert!(deque.len() == 2, "{}", deque.len());

        assert!(deque.push_front(3).is_ok());
        assert!(deque.len() == 3, "{}", deque.len());

        assert!(deque.push_front(4).is_err());
        assert!(deque.len() == 3, "{}", deque.len());
    }

    #[test]
    fn forward_rotations_work() {
        let mut deque: ArrDeque<usize, 3> = ArrDeque::default();
        for i in 0..3 {
            assert!(deque.push_back(i).is_ok());
        }

        for _ in 0..2 {
            assert!(deque.pop_front().is_some());
        }

        assert!(deque.push_back(3).is_ok());
        let mut iter = deque.iter();
        for i in 0..2 {
            println!("{}", i);
            assert_eq!(i + 2, *iter.next().unwrap());
        }
        assert!(iter.next().is_none());

        let mut iter_mut = deque.iter_mut();
        for i in 0..2 {
            assert_eq!(i + 2, *iter_mut.next().unwrap());
        }
        assert!(iter_mut.next().is_none());

        let mut into_iter = deque.into_iter();
        for i in 0..2 {
            assert_eq!(i + 2, into_iter.next().unwrap());
        }
        assert!(into_iter.next().is_none());
    }

    #[test]
    fn simple_integration_test() {
        let mut deque: ArrDeque<usize, 3> = ArrDeque::default();
        for i in 0..3 {
            assert!(deque.push_back(i).is_ok());
        }

        for (expected, actual) in deque.iter().enumerate() {
            assert_eq!(expected, *actual);
        }

        for (expected, actual) in deque.iter_mut().enumerate() {
            assert_eq!(expected, *actual);
        }

        for (expected, actual) in deque.into_iter().enumerate() {
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn complex_integration_test() {
        let mut deque: ArrDeque<usize, 3> = ArrDeque::default();
        // fill up the deque
        for _ in 0..3 {
            assert!(deque.push_back(0).is_ok());
        }

        // Do a bunch of rotations forwards
        for _ in 0..100 {
            assert!(deque.pop_front().is_some());
            assert!(deque.push_back(0).is_ok());
        }

        // Do a bunch of rotations backwards
        for _ in 0..98 {
            assert!(deque.pop_back().is_some());
            assert!(deque.push_front(0).is_ok());
        }

        // Do a bunch of front replacements
        for _ in 0..100 {
            assert!(deque.pop_front().is_some());
            assert!(deque.push_front(0).is_ok());
        }

        // Do a bunch of back replacements
        for _ in 0..100 {
            assert!(deque.pop_back().is_some());
            assert!(deque.push_back(0).is_ok());
        }

        deque.clear();
        assert!(deque.push_back(0).is_ok());
        assert!(deque.push_back(1).is_ok());

        let mut iter = deque.iter();
        for i in 0..2 {
            assert_eq!(i, *iter.next().unwrap());
        }
        assert!(iter.next().is_none());

        let mut iter_mut = deque.iter_mut();
        for i in 0..2 {
            assert_eq!(i, *iter_mut.next().unwrap());
        }
        assert!(iter_mut.next().is_none());

        let mut into_iter = deque.into_iter();
        for i in 0..2 {
            assert_eq!(i, into_iter.next().unwrap());
        }
        assert!(into_iter.next().is_none());
    }
}
