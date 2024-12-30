use {super::process::Process, array_macro::array};

const MAX_ALLOWED_PROCESSES: usize = 64;

pub struct ProcessManager {
  processes: [Process; MAX_ALLOWED_PROCESSES],
  initProcess: *mut Process,
}

impl ProcessManager {
  pub const fn new() -> Self {
    Self {
      processes: array![_ => Process::new( ); MAX_ALLOWED_PROCESSES],
      initProcess: 0 as *mut Process,
    }
  }
}

const PROCESS_MANAGER: ProcessManager = ProcessManager::new();
