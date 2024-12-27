use {super::buddy::BuddyAllocator, core::alloc::GlobalAlloc};

const DRAM_STARTING_ADDRESS: usize = 0x80000000;
const DRAM_SIZE: usize = 256 * 1024 * 1024; // (256 MB).
const DRAM_ENDING_ADDRESS: usize = DRAM_STARTING_ADDRESS + DRAM_SIZE;

pub struct ArnoAllocator {
  buddyAllocator: BuddyAllocator,
}

impl ArnoAllocator {
  pub const fn new() -> Self {
    Self {
      buddyAllocator: BuddyAllocator::new(),
    }
  }

  // Initializes the underlying Buddy allocator.
  pub unsafe fn init(&mut self) {
    println!("INFO : Initializing Arno allocator");

    // Determine where the loaded Kernel code has ended.
    extern "C" {
      fn _kernelEndAddress(); // This function pointer points to the _kernelEndAddress linker symbol.
    }
    let kernelEndAddress = _kernelEndAddress as usize;
    println!(
      "DEBUG : Loaded Kernel code ends at : {:#x}",
      kernelEndAddress
    );

    self.buddyAllocator.init(
      kernelEndAddress,
      DRAM_ENDING_ADDRESS,
      16,   // Leaf size = 16 bytes.
      4096, // Max alignment size = 4 KB.
    );
  }
}

unsafe impl GlobalAlloc for ArnoAllocator {
  unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
    unimplemented!()
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
    unimplemented!()
  }
}
