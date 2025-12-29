Returns a raw pointer to the [`GrowLock`]'s buffer, or a dangling
raw pointer valid for zero sized reads if the [`GrowLock`] didn't
allocate.

The caller must ensure that the [`GrowLock`] outlives the pointer returned by
this function, or else it will end up dangling.

The caller must also ensure that the memory this pointer points to is never
written to using this pointer or any pointer derived from it. If you want to
manually grow the [`GrowLock`], use [`GrowLock::as_mut_ptr`].

Unlike [`Vec::as_ptr`], modifying the [`GrowLock`] will never reallocate
and so the pointer will be valid as long as the [`GrowLock`] also is.

