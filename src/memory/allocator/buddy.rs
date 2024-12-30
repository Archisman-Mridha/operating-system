use {
  super::{
    list::FreeList,
    utils::{self, ceilToMultiple, initSliceWith0s},
  },
  core::{alloc::Layout, cmp::max, mem::MaybeUninit, ptr},
};

/*
  NOTE : We're using raw pointers for slices everywhere, to disable bound checks while accessing
         slice elements. This results to a faster performance.

  REFER : https://youtu.be/DRAHRJEAEso and ./BuddyAllocationAlgorithm.pdf.
*/
pub struct BuddyAllocator {
  isInitialized: bool,

  effectiveMemoryRegionStarting: usize,
  effectiveMemoryRegionEnding: usize,

  leafSize: usize, // Smallest chunk size.
  maxAlignmentSize: usize,

  // Number of possible chunk sizes / chunk classes (C_rs) we can have in the given effective memory
  // region.
  // Represented by the set S in the PDF document.
  chunkClassesCount: usize,
  chunkClasses: MaybeUninit<*mut [ChunkClass]>,
}

impl BuddyAllocator {
  pub const fn new() -> Self {
    Self {
      isInitialized: false,

      effectiveMemoryRegionStarting: 0,
      effectiveMemoryRegionEnding: 0,

      leafSize: 0,
      maxAlignmentSize: 0,

      chunkClassesCount: 0,
      chunkClasses: MaybeUninit::uninit(),
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

    let mut pointer = utils::ceilToMultiple(memoryRegionStarting, max(leafSize, maxAlignmentSize));

    // Determine the effective memory region.
    // This is represented by M' in the PDF document.
    {
      self.effectiveMemoryRegionStarting = pointer;

      self.effectiveMemoryRegionEnding =
        utils::floorToMultiple(memoryRegionEnding, max(leafSize, maxAlignmentSize));

      println!(
        "DEBUG : Effective memory region : {} - {}",
        self.effectiveMemoryRegionStarting, self.effectiveMemoryRegionEnding,
      );
    }

    let effectiveMemoryRegionSize =
      self.effectiveMemoryRegionEnding - self.effectiveMemoryRegionStarting;

    self.leafSize = leafSize;
    self.maxAlignmentSize = maxAlignmentSize;

    self.chunkClassesCount = (usize::ilog2(effectiveMemoryRegionSize / leafSize) + 1) as usize;

    // Initialize self.chunkClasses with 0s.
    let chunkClasses = initSliceWith0s::<ChunkClass>(&mut pointer, self.chunkClassesCount);
    self.chunkClasses.as_mut_ptr().write(chunkClasses);

    // For each possible chunk size, initialize the BitMaps BM and sBM, in the corresponding chunk
    // class C_r.
    for k_r in 0..self.chunkClassesCount {
      let chunkClassSize = self.getChunkClassSize(k_r);
      let chunkClass = self.getChunkClass(k_r);

      let bitMapByteSize = ceilToMultiple(chunkClassSize, 8) / 8;

      // TODO : initialize chunkClass.freeList.

      // Initialize BM.
      let bm = initSliceWith0s::<u8>(&mut pointer, bitMapByteSize);
      chunkClass.bm.as_mut_ptr().write(bm);

      // Initialize sBM.
      {
        // The leaf chunks cannot be split further. So, we'll not create an sBM for the chunk class
        // containining leaf chunks.
        if k_r == 0 {
          continue;
        }

        let sBM = initSliceWith0s::<u8>(&mut pointer, bitMapByteSize);
        chunkClass.sBM.as_mut_ptr().write(sBM);
      }
    }

    unimplemented!();

    self.isInitialized = true;
  }

  fn initFreeRegions(&mut self, pointer: usize) {
    let k_max = self.chunkClassesCount - 1;

    for k_r in 0..k_max {}
  }
}

impl BuddyAllocator {
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

  pub fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) -> *mut u8 {
    unimplemented!()
  }
}

// Holds information about all possible chunks having the same size (say r).
// Represented by the set C_r in the PDF document.
#[repr(C)]
struct ChunkClass {
  freeList: FreeList,

  // Represented by the BitMap BM_r in the PDF document.
  // The ith bit indicates whether the ith possible chunk in C_r has been allocated or not.
  bm: MaybeUninit<*mut [u8]>,

  // Represented by the BitMap sBM_r in the PDF document.
  // The ith bit indicates whether the ith possible chunk in C_r has been split or not.
  sBM: MaybeUninit<*mut [u8]>,
}

impl BuddyAllocator {
  // Returns the chunk C_r corresponding the given k_r.
  //
  // SAFETY : Self.init( ) must have been invoked before.
  unsafe fn getChunkClass(&mut self, k_r: usize) -> &mut ChunkClass {
    let chunkClasses = *self.chunkClasses.as_ptr();
    chunkClasses.get_unchecked_mut(k_r).as_mut().unwrap()
  }

  // Returns size of the chunk class C_r corresponding to the given k_r.
  // The size of the chunk class C_r represents the maximum possible count of chunks (with the given
  // size) we can have in the effective memory region.
  // Represented by the formula (4) in the PDF document.
  #[inline]
  fn getChunkClassSize(&self, k_r: usize) -> usize {
    1 << ((self.chunkClassesCount - 1) - k_r)
  }

  // Returns the size of a chunk belonging to the chunk class C_r corresponding to the given k_r.
  // Derived from the equation after (3) in the PDF document.
  fn getChunkSize(&self, k_r: usize) -> usize {
    self.leafSize * (1 << k_r)
  }

  // Returns the memory address of the ith member chunk of the chunk class C_r corresponding to the
  // given k_r.
  // Represented by the formula (6) in the PDF document.
  fn getChunkAddress(&self, k_r: usize, i: usize) -> usize {
    self.effectiveMemoryRegionStarting + (i * self.getChunkSize(k_r))
  }

  // Returns index of the chunk with the given memory address, in the chunk class corresponding to
  // the given k_r.
  fn getChunkIndexInChunkClassFromAddress(&self, address: usize, k_r: usize) -> usize {
    (address - self.effectiveMemoryRegionStarting) / self.getChunkSize(k_r)
  }
}
