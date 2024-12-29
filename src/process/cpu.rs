use {
  super::core::Core,
  crate::arch::riscv::{qemu::MAX_CORES, registers::tp::Tp},
  array_macro::array,
};

pub struct _CPU([Core; MAX_CORES]);

impl _CPU {
  pub const fn new() -> Self {
    Self(array![_ => Core::new( ); MAX_CORES])
  }

  // Returns the CPU core on which the invoker is running.
  pub unsafe fn getCurrentCore(&mut self) -> &mut Core {
    let hartID = Tp.read();
    &mut self.0[hartID]
  }
}

pub static mut CPU: _CPU = _CPU::new();
