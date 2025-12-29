use {
    crate::{AtomicVec, error::VecFull},
    std::{
        alloc::{Allocator, Global},
        ops,
        sync::{MutexGuard, atomic::Ordering},
    },
};

pub struct AtomicVecGuard<'a, T, A: Allocator = Global> {
    pub(crate) _guard: MutexGuard<'a, ()>,
    pub(crate) vec: &'a AtomicVec<T, A>,
}

impl<T, A: Allocator> ops::Deref for AtomicVecGuard<'_, T, A> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T, A: Allocator> AtomicVecGuard<'_, T, A> {
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        self.vec.as_slice()
    }
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    #[inline]
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.vec.capacity()
    }
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.vec.len()
    }
    /// # Panics
    /// Panics if `self.is_full()`.
    pub fn push(&mut self, value: T) {
        // We locked the mutex so writes cannot happen.
        let len = self.vec.len.load(Ordering::Relaxed);
        let cap = self.capacity();

        assert!(len < cap, "length overflow");

        // SAFETY: the ptr is still in the allocated block, even after add(len)
        unsafe {
            let dst = self.vec.as_non_null_ref().add(len);
            dst.write(value);
            self.vec.len.store(len + 1, Ordering::Release);
        }
    }
    /// # Errors
    /// Returns an error if `self.is_full()`.
    pub fn try_push(&mut self, value: T) -> Result<(), VecFull> {
        // We locked the mutex so writes cannot happen.
        let len = self.vec.len.load(Ordering::Relaxed);
        let cap = self.vec.capacity();

        if len >= cap {
            return Err(VecFull);
        }

        // SAFETY: the ptr is still in the allocated block, even after add(len)
        unsafe {
            let dst = self.vec.as_non_null_ref().add(len);
            dst.write(value);
        }
        self.vec.len.store(len + 1, Ordering::Release);

        Ok(())
    }
}

impl<T, A: Allocator> Extend<T> for AtomicVecGuard<'_, T, A> {
    /// Extends the [`AtomicVec<T>`] with the contents of an iterator.
    /// 
    /// # Panics
    /// This panics if the iterator has more elements than `self.capacity() -
    /// self.len()` (i.e. pushing all the elements would overflow
    /// `self.capacity()`.
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        for elem in iter {
            self.push(elem);
        }
    }
}
