use crate::{memory::allocator::GLOBAL_ALLOCATOR, println};

#[no_mangle]
pub unsafe extern "C" fn main() {
  println!("DEBUG : Switched to Supervisor mode and jumped to main");

  // Initialize the physical memory allocator.
  GLOBAL_ALLOCATOR.init();
}
