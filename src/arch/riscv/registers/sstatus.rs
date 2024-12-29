use core::arch::asm;

#[allow(non_camel_case_types)]
enum BitMasks {
  SSTATUS_SIE_CLEARER = 0 << 1,
}

// The sstatus (Supervisor Status) register, keeps track of the processor’s current operating state.
// REFER : section 10.1.1 in privileged ISA manual.
pub struct Sstatus;

// The SIE bit enables or disables all interrupts in supervisor mode. When SIE is clear, interrupts
// are not taken while in supervisor mode.
// When the hart is running in user-mode, the value in SIE is ignored, and supervisor-level
// interrupts are enabled. The supervisor can disable individual interrupt sources using the sie
// CSR.
impl Sstatus {
  // Returns whether all interrupts are disable or not.
  #[inline]
  pub unsafe fn areInterruptsEnabled(&self) -> bool {
    let mut bits: usize;
    asm!("csrr {}, sstatus", out(reg)bits);

    (bits & BitMasks::SSTATUS_SIE_CLEARER as usize) != 1
  }

  // Disable all interrupts by clearing the SIE bits.
  #[inline]
  pub unsafe fn disableInterrupts(&self) {
    asm!("csrc sstatus, {}", in(reg)BitMasks::SSTATUS_SIE_CLEARER as usize);
  }
}
