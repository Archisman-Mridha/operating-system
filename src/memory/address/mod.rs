use crate::arch::riscv::qemu::PAGE_SIZE;

pub trait Address {
  fn new(value: usize) -> Self;

  fn asUsize(&self) -> usize;
  fn asUsizeRef(&self) -> &usize;
  fn asUsizeMutRef(&mut self) -> &mut usize;

  // Increases this address by the size of a page.
  #[inline]
  fn increaseByAPage(&mut self) {
    *self.asUsizeMutRef() += PAGE_SIZE;
  }
}

pub mod physical;
pub mod r#virtual;
