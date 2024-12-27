#![allow(non_snake_case)]
#![allow(special_module_name)]
#![allow(clippy::upper_case_acronyms)]
#![feature(slice_ptr_get)]
//
// Rust's standard library depends on libc, which in-turn depends on the underlying Operating
// System. Since we're building the Operating System itself, we cannot use the standard library.
#![no_std]
/*
  (1) In a typical Rust binary that links the standard library, execution starts in a C runtime
    library called crt0 (C runtime 0), which sets up the environment for a C application. This
    includes creating a stack and placing the arguments in the right registers.

  (2) The C runtime then invokes the entry point of the Rust runtime, which is marked by the start
    language item. Rust only has a very minimal runtime, which takes care of some small things such
    as setting up stack overflow guards or printing a backtrace on panic.

  (3) The runtime then finally calls the main function.

  Our freestanding executable doesn't have the crt0 and Rust runtime. The ./asm/entry.S file invokes
  the start( ) function.
*/
#![no_main]

use registers::pmp;

core::arch::global_asm!(include_str!("asm/entry.S"));

#[no_mangle] // Disabling name mangling to ensure that the Rust compiler really outputs a function
             // with the name start. Without the attribute, the compiler would generate some
             // cryptic symbol to give every function a unique name.
unsafe fn start() -> ! {
  use {
    core::arch::asm,
    main::main,
    registers::{
      medeleg::Medeleg, mepc::Mepc, mhartid::Mhartid, mideleg::Mideleg, mstatus::Mstatus,
      satp::Satp, sie::Sie, tp::Tp,
    },
  };

  println!("INFO : Kernel is starting....");

  Mstatus.setMppBitsToSMode();

  Mepc.set(main as usize);

  Satp.disablePaging();

  // Delegate traps (exceptions and interrupts) to the S-mode.
  Medeleg.delegateExceptionsToSMode();
  Mideleg.delegateInterruptsToSMode();

  // Enable interrupts for S-mode.
  Sie.enableInterrupts();

  // Give access of the complete Physical Memory to the S-mode.
  pmp::Pmpaddr0.defineFullPhysicalMemoryAsRegion();
  pmp::PmpCfg0.setPmpaddrConfig(
    0,
    pmp::AddressMatchingMode::TOR,
    pmp::PermissionLevel::RWX,
    false,
  );

  // Store the hardware-thread id in the tp (thread pointer) register.
  // TODO : Why do we need the tp register, if we already have the mhartid register?
  let hartId = Mhartid.read();
  Tp.write(hartId);

  // An MRET instruction is used to return from a trap in M-mode.
  // We return to S-mode (stored in MPP bits of the mstatus register) and start execution from
  // main( ) (whose address is stored in the mepc register).
  // REFER : section 3.1.6.1 in privileged ISA manual.
  asm!("mret");

  #[allow(clippy::empty_loop)]
  loop {}
}

#[macro_use]
mod console;
mod main;
mod modes;
mod registers;

extern crate alloc;
mod allocator;

// The panic_handler attribute defines the function that the compiler should invoke when a panic
// occurs. The standard library provides its own panic handler function, but in a no_std environment
// we need to define it ourselves.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
  // TODO : Print the panic message.
  println!("ERROR : Kernel panic occurred!");
  loop {}
}

// Language items are special functions and types that are required internally by the compiler.
// The eh_personality language item marks a function that is used for implementing stack unwinding.
// By default, Rust uses unwinding to run the destructors of all live stack variables in case of a
// panic. This ensures that all used memory is freed and allows the parent thread to catch the
// panic and continue execution.
// Unwinding, however, is a complicated process and requires some OS-specific libraries (e.g.
// libunwind on Linux), so we'll disable it, by setting panic = "abort" in Cargo.toml.
