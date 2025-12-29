Returns a raw pointer to the [`GrowLock`]'s buffer, or a dangling
raw pointer valid for zero sized reads if the [`GrowLock`] didn't
allocate.

The caller must ensure that the [`GrowLock`] outlives the pointer returned by
this function, or else it will end up dangling.

The caller must also ensure that all bytes from this pointer to
`size_of::<T>() * self.len()` remains unchanged.

Unlike [`Vec::as_mut_ptr`], modifying the [`GrowLock`] will never reallocate
and so the pointer will be valid as long as the [`GrowLock`] also is.

