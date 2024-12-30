use core::arch::asm;

/*
  The satp (Supervisor Address Translation and Protection) registers controls supervisor-mode
  address translation and protection. The CSR holds :

    (1) the Physical Page Number (PPN) of the root page table, i.e., its supervisor physical
        address divided by 4 KiB.

    (2) an Address Space Identifier (ASID), which facilitates address-translation fences on a
        per-address-space basis.

    (3) the MODE field, which selects the current address-translation scheme.

  The satp register is considered active when the effective privilege mode is S-mode or U-mode.

  REFER : section 10.1.11 in privileged ISA manual.
*/
pub struct Satp;

impl Satp {
  // Disales Virtual Address Translation (VAT), by setting the MODE to BARE.
  // Supervisor virtual addresses will then be equal to supervisor physical addresses, and there
  // will be no additional memory protection beyond the physical memory protection scheme described
  // in Section 3.7.
  // All the bits of the CSR, must be set to 0 to set MODE = BARE.
  #[inline]
  pub unsafe fn disableVirtualAddressTranslation(&self) {
    asm!("csrw satp, {}", in(reg)0);
  }
}
