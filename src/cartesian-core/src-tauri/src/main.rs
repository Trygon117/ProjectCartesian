#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod lobotomy;
use std::{sync::{Arc, Mutex}, thread, time};
use tauri::Manager; // Required for emitting events

struct CartesianState {
    monitor: Mutex<lobotomy::SystemMonitor>,
}

fn main() {
    let monitor = lobotomy::SystemMonitor::new();
    let state = Arc::new(CartesianState {
        monitor: Mutex::new(monitor),
    });

    let background_state = state.clone();

    tauri::Builder::default()
    .manage(state)
    .setup(move |app| {
        // 1. Clone the AppHandle (The Walkie-Talkie)
        // We need this to send events from the background thread to the UI
        let app_handle = app.handle();

        thread::spawn(move || {
            println!("[CORE] Background Monitor Thread Started");
            let target = "firefox";

            loop {
                if let Ok(mut guard) = background_state.monitor.lock() {
                    let result = guard.find_process(target);

                    // 2. Emit the Event
                    // Event Name: "process-update"
                    // Payload: The PID (or "0" if missing) - sending as String for simplicity
                    let payload = match result {
                        Some(pid) => pid.to_string(),
                      None => "0".to_string(),
                    };

                    // Send to the UI
                    let _ = app_handle.emit_all("process-update", payload);
                }

                thread::sleep(time::Duration::from_secs(1));
            }
        });
        Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
