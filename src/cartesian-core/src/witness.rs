use std::fs::OpenOptions;
use std::path::Path;
use memmap2::MmapMut;
use crate::config;
use image::{DynamicImage, ImageBuffer, Rgba};

const OFF_STATUS: usize = 0;
const OFF_WIDTH: usize = 4;
const OFF_HEIGHT: usize = 8;
const OFF_FRAME_ID: usize = 16;
const OFF_PIXELS: usize = 24;

const STATUS_WRITING: u8 = 0;

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

impl VisualCortex {
    /// Convert raw BGRA/RGBA bytes to a Rust Image
    pub fn to_dynamic_image(&self) -> Option<DynamicImage> {
        // Construct an ImageBuffer from the raw bytes
        // Note: Linux DMA-BUF is often BGRA. If colors are swapped, change Rgba to Bgra.
        ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(self.width, self.height, self.data.clone())
            .map(DynamicImage::ImageRgba8)
    }
}

impl Eye {
    pub fn new() -> Self {
        Self {
            mmap: None,
            last_frame_id: 0,
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        let path_str = config::get_shm_path();
        let path = Path::new(&path_str);
        
        if !path.exists() {
             if cfg!(target_os = "windows") {
                 return Ok(()); 
             }
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

    pub fn observe(&mut self) -> Option<VisualCortex> {
        if self.mmap.is_none() {
            let _ = self.connect();
            return None;
        }

        let mmap = self.mmap.as_ref().unwrap();

        if mmap[OFF_STATUS] == STATUS_WRITING {
            return None;
        }

        let width = u32::from_le_bytes(mmap[OFF_WIDTH..OFF_WIDTH+4].try_into().unwrap());
        let height = u32::from_le_bytes(mmap[OFF_HEIGHT..OFF_HEIGHT+4].try_into().unwrap());
        let frame_id = u64::from_le_bytes(mmap[OFF_FRAME_ID..OFF_FRAME_ID+8].try_into().unwrap());

        if frame_id <= self.last_frame_id {
            return None; 
        }

        let expected_size = (width * height * 4) as usize;
        let end = OFF_PIXELS + expected_size;
        
        if mmap.len() < end { return None; }

        let data = mmap[OFF_PIXELS..end].to_vec();
        self.last_frame_id = frame_id;

        Some(VisualCortex { width, height, data })
    }
}