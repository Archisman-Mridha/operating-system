pub const MAX_DISKS: usize = 10;

// A Disk is considered as a numbered sequence of Blocks.
// All reads and writes done by the kernel against the disk, are done in units of block.
//
// On the other hand, all reads and writes done by the disk itself, are done in a unit called
// sector. And usually, the sector size < block size.
//
// In UNIX systems for example, block size = 4096 and sector size = 512.
pub const BLOCK_SIZE: usize = 1024;

// Maximum number of blocks aa File System (FS) operation can write to.
pub const FS_OP_MAX_BLOCK_WRITES: usize = 10;

pub mod bcache;
pub mod disk;
