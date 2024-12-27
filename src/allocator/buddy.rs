use {
  crate::allocator::utils,
  core::{alloc::Layout, cmp::max, ptr},
};

// REFER : https://youtu.be/DRAHRJEAEso.
pub struct BuddyAllocator {
  isInitialized: bool,

  effectiveMemoryRegionStarting: usize,
  effectiveMemoryRegionEnding: usize,

  leafSize: usize, // Smallest chunk size.
  maxAlignmentSize: usize,

  // Order (element count) of the set S, containing all possible chunk sizes.
  // Refer to the derivation provided in ./BuddyAllocationAlgorithm.pdf.
  possibleChunkSizesCount: usize,
}

impl BuddyAllocator {
  pub const fn new() -> Self {
    Self {
      isInitialized: false,

      effectiveMemoryRegionStarting: 0,
      effectiveMemoryRegionEnding: 0,

      leafSize: 0,
      maxAlignmentSize: 0,

      possibleChunkSizesCount: 0,
    }
  }

  pub unsafe fn init(
    &mut self,
    memoryRegionStarting: usize,
    memoryRegionEnding: usize,
    leafSize: usize,
    maxAlignmentSize: usize,
  ) {
    // Assertions.
    {
      assert!(
        !self.isInitialized,
        "Buddy allocator is already initialized"
      );

      assert!(
        (memoryRegionEnding - memoryRegionStarting) > leafSize,
        "Leaf size if greater than the memory size"
      );

      assert!(leafSize.is_power_of_two(), "Leaf size is not a power of 2");

      assert!(
        maxAlignmentSize.is_power_of_two(),
        "Max alignment size is not a power of 2"
      );
    }

    println!("DEBUG : Initializing Buddy allocator");

    // Determine the effective memory region.
    {
      self.effectiveMemoryRegionStarting =
        utils::ceilToMultiple(memoryRegionStarting, max(leafSize, maxAlignmentSize));

      self.effectiveMemoryRegionEnding =
        utils::ceilToMultiple(memoryRegionEnding, max(leafSize, maxAlignmentSize));

      println!(
        "DEBUG : Effective memory region : {} - {}",
        self.effectiveMemoryRegionStarting, self.effectiveMemoryRegionEnding
      );
    }

    self.leafSize = leafSize;
    self.maxAlignmentSize = maxAlignmentSize;

    // Refer to the derivation provided in ./BuddyAllocationAlgorithm.pdf.
    let effectiveMemoryRegionSize =
      self.effectiveMemoryRegionEnding - self.effectiveMemoryRegionStarting;
    self.possibleChunkSizesCount =
      (usize::ilog2(effectiveMemoryRegionSize / leafSize) + 1) as usize;

    for possibleChunkSize in 0..self.possibleChunkSizesCount {}

    self.isInitialized = true;
  }

  pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
    // CASE : Allocating zero sized types.
    if layout.size() == 0 {
      return ptr::null_mut();
    }

    assert!(
      layout.align() <= self.maxAlignmentSize,
      "Memory layout alignment size is greater than max allowed alignment size"
    );

    unimplemented!()
  }
}
