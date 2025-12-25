use std::fs::OpenOptions;
use std::path::Path;
use memmap2::MmapMut;
use std::sync::atomic::{AtomicU8, Ordering};

/// THE WITNESS (Vision System)
/// Reads frames from the Shared Memory Ring Buffer created by the Root Daemon.

const SHM_PATH: &str = "/dev/shm/cartesian_eye";

// Offsets must match the Python/C++ Writer
const OFF_STATUS: usize = 0;
const OFF_WIDTH: usize = 4;
const OFF_HEIGHT: usize = 8;
const OFF_FRAME_ID: usize = 16;
const OFF_PIXELS: usize = 24;

const STATUS_WRITING: u8 = 0;
const STATUS_READY: u8 = 1;

pub struct Eye {
    mmap: Option<MmapMut>,
    last_frame_id: u64,
}

#[derive(Debug, Clone)]
pub struct VisualCortex {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Eye {
    pub fn new() -> Self {
        Self {
            mmap: None,
            last_frame_id: 0,
        }
    }

    /// Connect to the Ring Buffer
    pub fn connect(&mut self) -> Result<(), String> {
        let path = Path::new(SHM_PATH);
        if !path.exists() {
            return Err("Witness: SHM file not found.".to_string());
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| e.to_string())?;

        let mmap = unsafe { 
            MmapMut::map_mut(&file).map_err(|e| e.to_string())? 
        };

        self.mmap = Some(mmap);
        Ok(())
    }

    /// Poll for a new frame
    pub fn observe(&mut self) -> Option<VisualCortex> {
        if self.mmap.is_none() {
            let _ = self.connect();
            return None;
        }

        let mmap = self.mmap.as_ref().unwrap();

        // 1. Atomic Check (Is writer busy?)
        if mmap[OFF_STATUS] == STATUS_WRITING {
            return None;
        }

        // 2. Read Metadata
        let width = u32::from_le_bytes(mmap[OFF_WIDTH..OFF_WIDTH+4].try_into().unwrap());
        let height = u32::from_le_bytes(mmap[OFF_HEIGHT..OFF_HEIGHT+4].try_into().unwrap());
        let frame_id = u64::from_le_bytes(mmap[OFF_FRAME_ID..OFF_FRAME_ID+8].try_into().unwrap());

        if frame_id <= self.last_frame_id {
            return None; // No new data
        }

        // 3. Copy Data
        let expected_size = (width * height * 4) as usize;
        let end = OFF_PIXELS + expected_size;
        
        if mmap.len() < end { return None; }

        let data = mmap[OFF_PIXELS..end].to_vec();
        self.last_frame_id = frame_id;

        Some(VisualCortex { width, height, data })
    }
}