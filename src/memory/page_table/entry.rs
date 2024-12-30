use bitflags::bitflags;

pub enum BitMasks {
  MAKE_VALID = 1 << 0,

  MAKE_READABLE = 1 << 1,
  MAKE_WRITABLE = 1 << 2,
  MAKE_EXECUTABLE = 1 << 3,

  MAKE_USERSPACE_ACCESSIBLE = 1 << 4,
}

bitflags! {
  #[derive(Copy, Clone)]
  pub struct PTEBitFlags: usize {
    const V = BitMasks::MAKE_VALID as usize;

    const R = BitMasks::MAKE_READABLE as usize;
    const W = BitMasks::MAKE_WRITABLE as usize;
    const X = BitMasks::MAKE_EXECUTABLE as usize;

    const U = BitMasks::MAKE_USERSPACE_ACCESSIBLE as usize;
  }
}

// REFER : Figure 62 in page 111 for the bit structure of a Page Table Entry (PTE).
#[derive(Clone, Copy)]
pub struct PageTableEntry(pub usize);

impl PageTableEntry {
  // Returns whether the Page Table Entry (PTE) is valid or not, by checking the V bit.
  #[inline]
  pub fn isValid(&self) -> bool {
    (self.0 & BitMasks::MAKE_VALID as usize) > 0
  }

  // Returns the Physical Address (PA) to the correspinding Physical Page.
  #[inline]
  pub fn toPhysicalAddress(&self) -> usize {
    (self.0 >> 10) << 12
  }

  // Makes the Page Table Entry (PTE) point to the Physical Page correspoding to the given Physical
  // Address (PA).
  // Also, uses the given bit flags for the PTE.
  // NOTE : The V bit is turned on regardless of whatever value is passed via the bit flags.
  pub fn setPhysicalAddress(&mut self, physicalAddress: usize, bitFlags: PTEBitFlags) {
    self.0 = ((physicalAddress >> 12) << 10) | (bitFlags | PTEBitFlags::V).bits();
  }
}
