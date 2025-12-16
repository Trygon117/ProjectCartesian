use sysinfo::{Pid, System}; // Removed SystemExt, ProcessExt for v0.30+

pub struct SystemMonitor {
    sys: System,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
        }
    }

    pub fn find_process(&mut self, name: &str) -> Option<Pid> {
        // Refresh processes
        self.sys.refresh_processes();

        for (pid, process) in self.sys.processes() {
            if process.name().to_lowercase().contains(&name.to_lowercase()) {
                return Some(*pid);
            }
        }
        None
    }
}
