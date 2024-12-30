use {
  crate::{arch::riscv::registers::tp::Tp, process::core::Core},
  core::{
    cell::{Cell, UnsafeCell},
    hint::spin_loop,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
  },
};

/*
  A SpinLock is a lock that causes a thread trying to acquire it to simply wait in a loop (spin)
  while repeatedly checking whether the lock is available.

  All interrupts remain disabled between the period a CPU core acquires and releases a SpinLock.
  This prevents context switching, thus saving CPU cycles and eliminating any deadlock situations.
  REFER : https://youtu.be/gQdflOUZQvA?si=HPniduCxD15lDWuO.
*/
pub struct SpinLock<T> {
  // We're using UnsafeCell<T> for interior mutability.
  // An immutable reference to SpinLock<T> (which, at the same time, can be shared across multiple
  // threads safely), can provide us a mutable raw pointer to T.
  data: UnsafeCell<T>,

  isAcquired: AtomicBool,
  ownerCPUCoreID: Cell<isize>, // ID of the CPU core which has acquired the SpinLock.
                               // NOTE : -1 means the SpinLock is currently acuired by none.
}

impl<T> SpinLock<T> {
  pub const fn new(data: T) -> Self {
    Self {
      data: UnsafeCell::new(data),

      isAcquired: AtomicBool::new(false),
      ownerCPUCoreID: Cell::new(-1),
    }
  }

  // Acquires the SpinLock.
  // Returns an immutable reference to the SpinLock.
  pub fn acquire(&self) -> SpinLockGuard<'_, T> {
    Core::enterInterruptsDisabledSection();

    // Ensure that the current CPU core isn't already owning the SpinLock.
    assert!(
      !self.isCurrentCPUCoreHolding(),
      "Current CPU core is already holding the SpinLock"
    );

    /*
      Steps of approaching to this solution :

      (1) while self.isLocked.load(Ordering::Relaxed) {}

        There can be a scenario, where both threads A and B exitted the while loop at the same time
        and they both acquire the lock as well, resulting to an undefined behaviour.

      (2) Removing the race condition by using compare_exchange :

          while self.isLocked.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                  .is_err()
          {
            // compare_exchange is an expensive operation, since all the threads need to coordinate
            // among each other (to understand in detail, have a look at the MESI protocol).
            // So only when the lock is released, we'll execute the next compare_exchange operation.
            while self.isLocked.load(Ordering::Relaxed) {
              // Hint to decrease the CPU core efficiency
              spin_loop();
            }
          }

      (3) compare_exchange is only allowed to fail if the actual value doesn't match the desired
          value. But, compare_exchange_weak is allowed to fail even if that holds true.

          On Intel / AMD x86_64, we have the compare and swap (CAS) instruction, because of which the
          compare_exchange opertion has low overhead.
          But on ARM64, we don't have such equivalent instruction. Instead, we have the LDREX and
          STREX instructions. Using LDREX, a CPU core can get exclusive access to a memory location.
          And only if the CPU core still has exclusive access to that memory location, the STREX
          instruction will succeed. So in ARM64, compare_exchange is implemented using a loop
          comprising of the LDREX and STREX instructions.

          This means :

            while self.isLocked.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                    .is_err()

          ends up being a nested loop.

          We will thus, rather use compare_exchange_weak( ). On Intel / AMD x86_64, this is an atomic
          operation, but on ARM64, this is implemented using LDREX and STREX (without any loop).

      (4) We have used the Ordering::Acquire memory ordering to establish the happens-before
          relationship.

      REFER : https://youtu.be/rMGWeSjctlY?si=ySq7B0A2Ucp-r6WV.
    */
    while self
      .isAcquired
      .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
      .is_err()
    {
      while self.isAcquired.load(Ordering::Relaxed) {
        spin_loop();
      }
    }

    unsafe { self.ownerCPUCoreID.set(Tp.read() as isize) };

    SpinLockGuard(self)
  }

  // Returns whether the CPU core of the invoker is already holding the SpinLock or not.
  pub fn isCurrentCPUCoreHolding(&self) -> bool {
    self.isAcquired.load(Ordering::Relaxed)
      && (self.ownerCPUCoreID.get() == unsafe { Tp.read() as isize })
  }
}

unsafe impl<T> Send for SpinLock<T> where T: Send {}
unsafe impl<T> Sync for SpinLock<T> where T: Send {}

pub struct SpinLockGuard<'a, T>(&'a SpinLock<T>);

// Automatic dereference conversion from &SpinLockGuard<T> to &T.
impl<T> Deref for SpinLockGuard<'_, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.0.data.get() }
  }
}

// Automatic dereference conversion from &mut SpinLockGuard<T> to &mut T.
impl<T> DerefMut for SpinLockGuard<'_, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.0.data.get() }
  }
}

// Release the SpinLock when the SpinLockGuard is dropped.
impl<T> Drop for SpinLockGuard<'_, T> {
  fn drop(&mut self) {
    self.0.release()
  }
}

impl<T> SpinLock<T> {
  fn release(&self) {
    // Ensure that the current CPU core is already owning the SpinLock.
    assert!(
      self.isCurrentCPUCoreHolding(),
      "Current CPU core isn't already holding this SpinLock"
    );

    self.ownerCPUCoreID.set(-1);

    /*
      Any CPU core trying to acquire the SpinLock right after this moment, must see all the memory
      operations upto this store operation with Release memory ordering.
      We are establishing a happens-before relationship between this store operation with Release
      memory ordering and all the load operations (with Acquire memory ordering) occuring after
      this in other threads.

      Release memory ordering : When coupled with a store, all previous operations become ordered
                                before any load of this value with Acquire (or stronger) ordering.
                                In particular, all writes upto this one become visible to all
                                threads that perform an Acquire (or stronger) load of this value.
    */
    self.isAcquired.store(false, Ordering::Release);

    Core::exitInterruptsDisabledSection();
  }
}

unsafe impl<T> Send for SpinLockGuard<'_, T> where T: Send {}
unsafe impl<T> Sync for SpinLockGuard<'_, T> where T: Send + Sync {}
