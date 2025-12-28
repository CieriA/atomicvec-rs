//! A fixed-capacity [`Vec`] which allows concurrences reads and
//! spin-lock writes.
//!
//! [`AtomicVec`] is designed for situations where reads need to
//! be extremely fast and cannot be blocked by writes. The
//! capacity is fixed and defined on creation, and cannot be
//! greater than [`isize::MAX`].
#![feature(allocator_api, sized_type_properties)]

mod cap;
pub mod error;
pub mod guard;
mod raw;
#[cfg(test)]
mod tests;

use {
    crate::{
        cap::Cap, error::TryReserveError, guard::AtomicVecGuard,
        raw::RawAtomicVec,
    },
    std::{
        alloc::{Allocator, Global},
        ops,
        ptr::NonNull,
        slice::{self, SliceIndex},
        sync::{
            Mutex,
            atomic::{AtomicUsize, Ordering},
        },
    },
};

/// A fixed-capacity [`Vec`] which allows concurrent reads and
/// spin-lock writes.
pub struct AtomicVec<T, A: Allocator = Global> {
    buf: RawAtomicVec<T, A>,
    len: AtomicUsize,
    mutex: Mutex<()>,
}

/// # Safety
/// If both `T` and `A` are [`Send`], it is safe to transfer an [`AtomicVec<T,
/// A>`] between threads as we have exclusive ownership of the buffer.
///
/// No thread can access the data while it's being moved.
unsafe impl<T: Send, A: Allocator + Send> Send for AtomicVec<T, A> {}
/// # Safety
/// If both `T` and `A` are [`Sync`], there's no interior mutability outside
/// the [`mutex`](Mutex) and the [`len`](AtomicUsize) (which is thread-safe).
///
/// All writes to the buffer are handled along the [`mutex`](Mutex), and so
/// this collection is [`Sync`]
unsafe impl<T: Send + Sync, A: Allocator + Sync> Sync for AtomicVec<T, A> {}

/// Getters for [`AtomicVec<T>`]
impl<T, A: Allocator> AtomicVec<T, A> {
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.buf.capacity()
    }
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len.load(Ordering::Acquire)
    }
    #[inline]
    #[must_use]
    pub const fn allocator(&self) -> &A {
        self.buf.allocator()
    }
    #[inline]
    #[must_use]
    pub const fn as_ptr(&self) -> *const T {
        self.buf.ptr()
    }
    #[inline]
    #[must_use]
    pub const fn as_mut_ptr(&self) -> *mut T {
        self.buf.ptr()
    }
    #[inline]
    #[must_use]
    pub const fn as_non_null(&self) -> NonNull<T> {
        self.buf.non_null()
    }
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY:
        // * `self.as_ptr()` is never null, and valid for reads up to
        //   `self.len()` if we can have a reference to `self` (which we do)
        // * the entire block of memory is within a single allocation
        // * at least `self.len()` number of elements are correctly initialized.
        // * `capacity * size_of::<T>()` doesn't overflow `isize::MAX`, so
        //   neither does `self.len() * size_of::<T>()`
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }
}

impl<T, A: Allocator> AtomicVec<T, A> {
    /// Constructs a new [`AtomicVec<T>`] in the provided allocator,
    /// returning an error if the allocation fails
    ///
    /// # Errors
    /// Returns an error if:
    /// * `cap * size_of::<T>` overflows [`isize::MAX`]
    /// * memory is exhausted
    ///
    /// # Examples
    /// ```
    /// #![feature(allocator_api)]
    /// use atomicvec::AtomicVec;
    /// use std::alloc::System;
    ///
    /// let my_atomic_vec = AtomicVec::try_new_in(10, System);
    /// ```
    pub fn try_new_in(
        capacity: usize,
        alloc: A,
    ) -> Result<Self, TryReserveError> {
        let Some(cap) = Cap::try_new::<T>(capacity) else {
            return Err(TryReserveError::CapacityOverflow);
        };
        let buf = RawAtomicVec::try_new_in(cap, alloc)?;

        Ok(Self {
            buf,
            len: AtomicUsize::new(0),
            mutex: Mutex::new(()),
        })
    }

    /// Constructs a new [`AtomicVec<T>`] in the provided allocator.
    ///
    /// # Examples
    /// ```
    /// #![feature(allocator_api)]
    /// use atomicvec::AtomicVec;
    /// use std::alloc::System;
    ///
    /// let my_atomic_vec = AtomicVec::new_in(10, System);
    /// ```
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new_in(capacity: usize, alloc: A) -> Self {
        let cap = Cap::try_new::<T>(capacity)
            .unwrap_or_else(|| panic!("{}", TryReserveError::CapacityOverflow));
        let buf = RawAtomicVec::new_in(cap, alloc);

        Self {
            buf,
            len: AtomicUsize::new(0),
            mutex: Mutex::new(()),
        }
    }

    // TODO: create an error for this (now it is an option)
    #[inline]
    pub fn lock(&self) -> Option<AtomicVecGuard<'_, T, A>> {
        let guard = self.mutex.lock().ok()?;

        Some(AtomicVecGuard {
            _guard: guard,
            vec: self,
        })
    }
}

impl<T> AtomicVec<T> {
    /// Constructs a new [`AtomicVec<T>`],
    /// returning an error if the allocation fails
    ///
    /// # Errors
    /// Returns an error if:
    /// * `cap * size_of::<T>` overflows `isize::MAX`
    /// * memory is exhausted
    ///
    /// # Examples
    /// ```
    /// use atomicvec::AtomicVec;
    ///
    /// let my_atomic_vec = AtomicVec::try_new(10);
    /// ```
    #[inline]
    pub fn try_new(capacity: usize) -> Result<Self, TryReserveError> {
        Self::try_new_in(capacity, Global)
    }

    /// Constructs a new [`RawAtomicVec<T>`].
    ///
    /// # Examples
    /// ```
    /// use atomicvec::AtomicVec;
    ///
    /// let my_atomic_vec = AtomicVec::new(10);
    /// ```
    #[inline]
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self::new_in(capacity, Global)
    }

    #[must_use]
    pub fn from_elem(capacity: usize, elem: T) -> Self
    where
        T: Copy,
    {
        let this = Self::new(capacity);
        let guard = this.lock().unwrap();
        for _ in 0..capacity {
            guard.push(elem);
        }
        drop(guard);
        this
    }
    #[must_use]
    pub fn from_default(capacity: usize) -> Self
    where
        T: Default,
    {
        let this = Self::new(capacity);
        let guard = this.lock().unwrap();
        for _ in 0..capacity {
            guard.push(T::default());
        }
        drop(guard);
        this
    }
}
/// FIXME: I don't know if this is sound
impl<T, A: Allocator> ops::Deref for AtomicVec<T, A> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

/// FIXME: I don't know if this is sound
impl<T, I, A> ops::Index<I> for AtomicVec<T, A>
where
    I: SliceIndex<[T]>,
    A: Allocator,
{
    type Output = <I as SliceIndex<[T]>>::Output;
    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        ops::Index::index(&**self, index)
    }
}
