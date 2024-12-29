use core::ptr::{slice_from_raw_parts_mut, write_bytes};

// Rounds up x to the multiple of n, which is greater than and closest to x.
#[inline]
pub(super) fn ceilToMultiple(x: usize, n: usize) -> usize {
  (((x - 1) / n) + 1) * n
}

// Rounds down x to the multiple of n, which is lesser than and closest to x.
#[inline]
pub(super) fn floorToMultiple(x: usize, n: usize) -> usize {
  (x / n) * n
}

// Initializes a slice (of type T and of the given size) starting from the memory address the given
// pointer is pointing to, with 0s.
// The pointer is moved to the ending of the initialized slice.
pub unsafe fn initSliceWith0s<T>(pointer: &mut usize, sliceLen: usize) -> *mut [T] {
  let startingAddress = *pointer as *mut T;
  *pointer += size_of::<T>() * sliceLen; // Update pointer to the end of the slice.

  // Write 0 from the starting to the ending address.
  write_bytes(startingAddress, 0, sliceLen);

  slice_from_raw_parts_mut(startingAddress, sliceLen)
}
