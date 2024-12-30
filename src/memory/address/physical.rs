use super::Address;

pub struct PhysicalAddress(pub usize);

impl Address for PhysicalAddress {
  fn new(value: usize) -> Self {
    Self(value)
  }

  #[inline]
  fn asUsize(&self) -> usize {
    self.0
  }

  #[inline]
  fn asUsizeRef(&self) -> &usize {
    &self.0
  }

  #[inline]
  fn asUsizeMutRef(&mut self) -> &mut usize {
    &mut self.0
  }
}
