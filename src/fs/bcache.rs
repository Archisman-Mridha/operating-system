use {
  super::{BLOCK_SIZE, FS_OP_MAX_BLOCK_WRITES},
  crate::locks::{
    sleeplock::{SleepLock, SleepLockGuard},
    spinlock::SpinLock,
  },
  array_macro::array,
  core::{
    ptr,
    sync::atomic::{AtomicBool, Ordering},
  },
};

// Maximum number of blocks we can store at any moment of time, in the Buffer Cache (LRUCache).
pub const BCACHE_SIZE: usize = FS_OP_MAX_BLOCK_WRITES * 3;

struct LRUCacheNode {
  // Index of this LRUCacheNode in the LRUCache.nodes array.
  index: usize,

  previous: *mut LRUCacheNode,
  next: *mut LRUCacheNode,

  // If the refCount is not 0, that means this Buffer Cache (LRUCache) node is being used.
  refCount: usize,

  // Corresponding disk and block numbers.
  diskNumber: usize,
  blockNumber: usize,
}

impl LRUCacheNode {
  pub const fn new() -> Self {
    Self {
      index: 0,

      previous: ptr::null_mut(),
      next: ptr::null_mut(),

      refCount: 0,

      diskNumber: 0,
      blockNumber: 0,
    }
  }
}

// This LRUCache is wrapped around by BCache and must be first initialized using BCache.init( ).
// NOTE : Implemented using a Doubly Linked List (not a Circular Linked List).
struct LRUCache {
  head: *mut LRUCacheNode,
  tail: *mut LRUCacheNode,

  nodes: [LRUCacheNode; BCACHE_SIZE],
}

impl LRUCache {
  const fn new() -> Self {
    Self {
      head: ptr::null_mut(),
      tail: ptr::null_mut(),

      nodes: array![_ => LRUCacheNode::new(); BCACHE_SIZE],
    }
  }

  // Looks for an LRUCache node corresponding to the given disk and block numbers combination.
  // If found, returns the index of the node in the LRUCache.nodes array and the current number of
  // users of this node (refCount).
  fn get(&self, diskNumber: usize, blockNumber: usize) -> Option<(usize, *mut usize)> {
    let mut nodeRawPointer = self.head;
    while !nodeRawPointer.is_null() {
      let node = unsafe { &mut *nodeRawPointer };

      if (node.diskNumber == diskNumber) && (node.blockNumber == blockNumber) {
        node.refCount += 1;
        return Some((node.index, &mut node.refCount));
      }

      nodeRawPointer = node.next;
    }
    None
  }

  // Recycle an unused LRUCache node (a node whose refCount = 0), with the given disk and block
  // numbers combintaion .
  //
  // If a recyclable BCache node is found, then returns its index in the LRUCache.nodes array and
  // the current number of places it's being used in (refCount - which will always be 1).
  //
  // NOTE : In this LRU cache, we're storing the least recently used (LRU) nodes towards the head.
  //				So, we will start searching from the tail towards the head.
  fn recycle(&self, diskNumber: usize, blockNumber: usize) -> Option<(usize, *mut usize)> {
    let mut nodeRawPointer = self.tail;
    while !nodeRawPointer.is_null() {
      let node = unsafe { &mut *nodeRawPointer };

      if node.refCount == 0 {
        node.diskNumber = diskNumber;
        node.blockNumber = blockNumber;

        node.refCount += 1;

        return Some((node.index, &mut node.refCount));
      }

      nodeRawPointer = node.previous;
    }
    None
  }

  // Decreases the refCount of the givnen node.
  // If the final refCount is 0, then that means the node has become the least recently used (LRU)
  // one and gets moved to the head.
  fn decreaseRefCount(&mut self, index: usize) {
    let node = &mut self.nodes[index];

    node.refCount -= 1;

    // The node has become the least recently used (LRU) one and needs to be moved to the head.
    if node.refCount == 0 && !ptr::eq(node, self.head) {
      // If this node is currently the tail, then set node.previous as the tail.
      if ptr::eq(node, self.tail) {
        self.tail = node.previous;
      }

      // Detach the node, from its previous and next.
      unsafe {
        node.previous.as_mut().unwrap().next = node.next;
        node.next.as_mut().unwrap().previous = node.previous;
      }

      // Put the node in the head.

      node.previous = ptr::null_mut();
      node.next = self.head;

      unsafe { self.head.as_mut().unwrap().previous = node };

      self.head = node;
    }
  }
}

unsafe impl Send for LRUCache {}

#[repr(C, align(8))]
struct BlockData([u8; BLOCK_SIZE]);

impl BlockData {
  pub const fn new() -> Self {
    Self([0; BLOCK_SIZE])
  }
}

struct BlockDataGuard {
  // Indicates whether the corresponding BlockData contains garbage or actual data from the disk.
  isValid: AtomicBool,

  blockData: SleepLock<BlockData>,
}

impl BlockDataGuard {
  pub const fn new() -> Self {
    Self {
      isValid: AtomicBool::new(false),
      blockData: SleepLock::new(BlockData::new()),
    }
  }
}

pub struct BCacheNode<'a> {
  index: usize, // = LRUCacheNode.index.

  refCount: *mut usize,

  diskNumber: usize,
  blockNumber: usize,

  blockData: SleepLockGuard<'a, BlockData>,
}

// NOTE : The BCache (specifically BCache.BlockDataGuards) is guarded using a SleepLock. So only
// 1 process can use the BCache at a time.
pub struct BCache {
  lruCache: SpinLock<LRUCache>,
  blockDataGuards: [BlockDataGuard; BCACHE_SIZE],
}

impl BCache {
  pub const fn new() -> Self {
    Self {
      lruCache: SpinLock::new(LRUCache::new()),
      blockDataGuards: array![_ => BlockDataGuard::new(); BCACHE_SIZE],
    }
  }

  // Initializes the BCache.
  // NOTE : Should only be called once when the Kernel is intializing.
  pub fn init(&self) {
    let mut lruCacheGuard = self.lruCache.acquire();

    // Initialize the head and tail nodes of the LRU Cache.
    lruCacheGuard.head = &mut lruCacheGuard.nodes[0];
    lruCacheGuard.tail = &mut lruCacheGuard.nodes[BCACHE_SIZE - 1];

    // Establish links between the LRU cache nodes.

    lruCacheGuard.nodes[0].previous = ptr::null_mut();
    lruCacheGuard.nodes[0].next = &mut lruCacheGuard.nodes[1];

    lruCacheGuard.nodes[BCACHE_SIZE - 1].previous = &mut lruCacheGuard.nodes[BCACHE_SIZE - 2];
    lruCacheGuard.nodes[BCACHE_SIZE - 1].next = ptr::null_mut();

    for i in 1..=(BCACHE_SIZE - 2) {
      lruCacheGuard.nodes[i].previous = &mut lruCacheGuard.nodes[i - 1];
      lruCacheGuard.nodes[i].next = &mut lruCacheGuard.nodes[i + 1];
    }

    // Set index field for each node in the LRUCache.nodes array.
    lruCacheGuard
      .nodes
      .iter_mut()
      .enumerate()
      .for_each(|(i, node)| node.index = i);
  }

  // Read the given block in the given disk, through this BCache.
  pub fn read(&self, diskNumber: usize, blockNumber: usize) -> BCacheNode<'_> {
    let lruCacheGuard = self.lruCache.acquire();

    match lruCacheGuard.get(diskNumber, blockNumber) {
      // A cached block correspnding to the given disk and block number combination has been found.
      Some((lruCacheNodeIndex, lruCacheNodeRefCount)) => BCacheNode {
        index: lruCacheNodeIndex,

        refCount: lruCacheNodeRefCount,

        diskNumber,
        blockNumber,

        blockData: self.blockDataGuards[lruCacheNodeIndex].blockData.acquire(),
      },

      // The block data corresponding to the given disk and block number combination isn't cached.
      // So, we'll first find the least recently used (LRU) unused cached block (refCount = 0) and
      // use that to cache the block data.
      None => {
        let (lruCacheNodeIndex, lruCacheNodeRefCount) = lruCacheGuard
          .recycle(diskNumber, blockNumber)
          .expect("No least recently used (LRU) unused cached block found to recycle");

        // TODO : write the block data to the cache.

        self.blockDataGuards[lruCacheNodeIndex]
          .isValid
          .store(true, Ordering::Relaxed);

        BCacheNode {
          index: lruCacheNodeIndex,

          refCount: lruCacheNodeRefCount,

          diskNumber,
          blockNumber,

          blockData: self.blockDataGuards[lruCacheNodeIndex].blockData.acquire(),
        }
      }
    }
  }

  // Decrease the refCount (number of users) for the given cached block.
  pub fn decreaseRefCount(&mut self, index: usize) {
    self.lruCache.acquire().decreaseRefCount(index);
  }
}

pub static BCACHE: BCache = BCache::new();
