#[allow(clippy::module_inception)]
mod allocator;
mod buddy;
mod list;
mod utils;

use allocator::ArnoAllocator;

// Route all default allocation requests to Arno allocator.
#[global_allocator]
pub static mut GLOBAL_ALLOCATOR: ArnoAllocator = ArnoAllocator::new();
