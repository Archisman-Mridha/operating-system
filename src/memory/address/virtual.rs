use {
  super::Address,
  core::ops::{Add, Sub},
};

// REFER : Figure 60 in page 111 for the bit structure of a Virtual Address (VA).
#[derive(PartialEq, Copy, Clone)]
pub struct VirtualAddress(pub usize);

impl Address for VirtualAddress {
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

impl VirtualAddress {
  // Returns the related Page Number (PN) in the given Page Table level, corresponding to this
  // Virtual Address (VA).
  // If level > 0, then the PPN points to the corresponding PTE in the lower level. Otherwise, it
  // points to the actual Physical Address (PA) this Virtual Address (VA) translates to.
  pub fn getCorrespondingPPN(&self, pageTableLevel: usize) -> usize {
    const PAGE_OFFSET_BIT_COUNT: usize = 12;
    const PAGE_NUMBER_BIT_COUNT: usize = 9;
    const PAGE_NUMBER_EXTRACTOR_BIT_MASK: usize = 0x1FF;

    (self.0 >> (PAGE_OFFSET_BIT_COUNT + pageTableLevel * PAGE_NUMBER_BIT_COUNT))
      & PAGE_NUMBER_EXTRACTOR_BIT_MASK
  }
}

impl Add for VirtualAddress {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self(self.0 + rhs.0)
  }
}

impl Sub for VirtualAddress {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    Self(self.0 - rhs.0)
  }
}
