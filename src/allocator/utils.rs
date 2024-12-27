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
