use sysinfo::{Pid, System, Signal};

pub struct SystemMonitor {
    sys: System,
}

impl SystemMonitor {
    pub fn new() -> Self {
        SystemMonitor {
            sys: System::new_all(),
        }
    }

    pub fn find_process(&mut self, target_name: &str) -> Option<Pid> {
        self.sys.refresh_processes();
        for process in self.sys.processes().values() {
            if process.name().to_lowercase().contains(&target_name.to_lowercase()) {
                return Some(process.pid());
            }
        }
        None
    }

    // New Capability: The "Freeze Ray"
    pub fn toggle_suspend(&mut self, pid: Pid, suspend: bool) {
        // We must refresh the specific PID to make sure it still exists
        if self.sys.refresh_process(pid) {
            if let Some(process) = self.sys.process(pid) {
                // FIXED: Changed Signal::Cont to Signal::Continue
                let signal = if suspend { Signal::Stop } else { Signal::Continue };

                // process.kill_with() returns true if the signal was sent successfully
                match process.kill_with(signal) {
                    Some(true) => println!("[KERNEL] Sent {:?} to PID {}", signal, pid),
                    _ => println!("[ERROR] Failed to send signal to PID {}", pid),
                }
            }
        }
    }
}
