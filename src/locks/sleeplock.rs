use {
  super::spinlock::SpinLock,
  core::{
    cell::{Cell, UnsafeCell},
    ops::{Deref, DerefMut},
  },
};

// Unlike Spin Locks, we can hold a Sleep Lock for a very long amount of time. While holding the
// lock, we can sleep and context-switch.
pub struct SleepLock<T> {
  data: UnsafeCell<T>,

  // Used to atomically release / try to acquire this SleepLock without any interruptions /
  // context-switches.
  spinLock: SpinLock<()>,

  isAcquired: Cell<bool>,
  ownerPID: usize, // ID of the process which is currently holding this Sleep Lock.
                   // NOTE : 0 means the SleepLock is currently acquired by none.
}

impl<T> SleepLock<T> {
  pub const fn new(data: T) -> Self {
    Self {
      spinLock: SpinLock::new(()),

      isAcquired: Cell::new(false),
      ownerPID: 0,

      data: UnsafeCell::new(data),
    }
  }

  pub fn acquire(&self) -> SleepLockGuard<T> {
    let spinLockGuard = self.spinLock.acquire();

    while self.isAcquired.get() {
      // TODO : Let the process go to sleep.
      todo!("Let the process go to sleep");
    }

    self.isAcquired.set(true);

    SleepLockGuard(self)
  }

  pub fn release(&self) {
    let spinLockGuard = self.spinLock.acquire();

    self.isAcquired.set(true);
    // TODO : Wake up all the other sleeping processes waiting to acquire this SleepLock.
  }
}

unsafe impl<T> Sync for SleepLock<T> {}

pub struct SleepLockGuard<'a, T>(&'a SleepLock<T>);

// Automatic dereference conversion from &SleepLockGuard<T> to &T.
impl<T> Deref for SleepLockGuard<'_, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.0.data.get() }
  }
}

// Automatic dereference conversion from &mut SleepLockGuard<T> to &mut T.
impl<T> DerefMut for SleepLockGuard<'_, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.0.data.get() }
  }
}

// Release the SpinLock when the SleepLockGuard is dropped.
impl<T> Drop for SleepLockGuard<'_, T> {
  fn drop(&mut self) {
    self.0.release()
  }
}
