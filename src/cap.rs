//! Capacity abstraction to permit its invariants.

/// Same as `T::IS_ZST`, but stable.
/// This is always [**const evaluated**][const-block].
///
/// [const-block]: https://doc.rust-lang.org/reference/expressions/block-expr.html#const-blocks
#[inline]
pub(crate) const fn is_zst<T>() -> bool {
    const { size_of::<T>() == 0 }
}

/// Representation of the `capacity`.
///
/// This always fits an `isize` as allocations can never be larger than
/// `isize::MAX`.
///
/// # Invariants
/// Inner value must be less or equal than `isize::MAX`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Cap(usize);
impl Cap {
    /// A `capacity` of zero. This means **unallocated**.
    ///
    /// The capacity for a `ZST` is always zero.
    pub(crate) const ZERO: Self = Self(0);

    /// Creates a new `capacity` without checking its invariants.
    ///
    /// If `T` is a `ZST`, this returns a capacity of zero.
    ///
    /// # Safety
    /// `cap` must be <= `isize::MAX`. It is immediate UB to call this a
    /// value that exceed `isize::MAX`.
    #[inline]
    pub(crate) const unsafe fn new_unchecked<T>(cap: usize) -> Self {
        // SAFETY: the safety condition is transferred to the caller
        unsafe { Self::new::<T>(cap).unwrap_unchecked() }
    }

    /// Creates a new `capacity` if it is <= `isize::MAX`.
    ///
    /// if `T` is a `ZST`, this returns a capacity of zero.
    #[inline]
    pub(crate) const fn new<T>(cap: usize) -> Option<Self> {
        const I_MAX: usize = isize::MAX as usize;
        match cap {
            _ if is_zst::<T>() => Some(Cap::ZERO),
            // SAFETY: `cap` is less or equal than isize::MAX
            ..=I_MAX => Some(Self(cap)),
            _ => None,
        }
    }
    /// Returns the `capacity` as a primitive value.
    #[inline]
    pub(crate) const fn get(self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zst() {
        assert!(is_zst::<()>());
        assert!(!is_zst::<String>());
        assert!(!is_zst::<u8>());
    }
    #[test]
    fn new_cap() {
        assert_eq!(Cap::new::<char>(17).map(Cap::get), Some(17));
        assert_eq!(Cap::new::<()>(42), Some(Cap::ZERO));
        assert_eq!(Cap::new::<u128>(0), Some(Cap::ZERO));
        assert_eq!(Cap::new::<[i32; 49]>(usize::MAX), None);
    }
}
