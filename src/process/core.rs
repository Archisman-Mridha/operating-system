use {super::cpu::CPU, crate::arch::riscv::registers::sstatus::Sstatus};

pub struct Core {
  /*
    Each time while entering an interrupts-disabled section, we :

      (1) if entering the outermost interrupts-disabled section, then store whether interrupts were
          enabled or not (before entering the interrupts-disabled section) in the intena variable.

      (2) increase the noff counter.

    When the CPU core leaves the interrupts-disabled section, we :

      (1) decrease the noff counter

      (2) if leaving the outermost interrupts-disabled section, then enable interrupts based on
          whether they were enabled or not before entering the outermost interrupts-disabled section
          (stored in the intena variable).

    This prevents premature enabling of interrupts when dealing with nested interrupts-disabled
    sections (like acquiring a SpinLock A and then acquiring another SpinLock A without releasing
    B).

    REFER : https://www.youtube.com/watch?v=gQdflOUZQvA.
  */
  noff: usize,
  intena: bool,
}

impl Core {
  pub const fn new() -> Self {
    Self {
      noff: 0,
      intena: false,
    }
  }

  /*
    Invoke this to enter an interrupts-disabled section. Does the following :

      (1) if entering the outermost interrupts-disabled section, store whether interrupts were
          enabled or not (before entering the interrupts-disabled section) in the intena variable.

      (2) if entering the outermost interrupts-disabled section, disable interrupts.

      (3) increase the noff counter.
  */
  pub fn enterInterruptsDisabledSection() {
    let core = unsafe { CPU.getCurrentCore() };

    // CASE : Entering the outermost interrupts-disabled section.
    if core.noff == 0 {
      // Store whether interrupts were enabled or not (before entering the interrupts-disabled
      // section) in the intena variable.
      let areInterruptsCurrentlyEnabled = unsafe { Sstatus.areInterruptsEnabled() };
      core.intena = areInterruptsCurrentlyEnabled;

      // Disable interrupts.
      unsafe { Sstatus.disableInterrupts() };
    }

    // Increase the noff counter.
    core.noff += 1;
  }

  /*
    Invoke this to leave the interrupts-disabled section. Does the following :

      (1) decreases the noff counter

      (2) if leaving the outermost interrupts-disabled section, then enables interrupts based on
          whether they were enabled or not before entering the outermost interrupts-disabled
          section (stored in the intena variable).
  */
  pub fn exitInterruptsDisabledSection() {
    let core = unsafe { CPU.getCurrentCore() };

    // Decreases the noff counter.
    core.noff -= 1;

    // CASE : Leaving the outermost interrupts-disabled section.
    if core.noff == 0 {
      // Enable interrupts if they were enabled before entering the outermost interrupts-disabled
      // section.
      if core.intena {
        unsafe { Sstatus.disableInterrupts() };
      }
    }
  }
}
