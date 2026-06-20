use memmap2::MmapMut;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

pub struct ShmBuffer {
    path: PathBuf,
    #[allow(dead_code)]
    file: File,
    mmap: MmapMut,
    size: usize,
}

impl ShmBuffer {
    pub fn create(session_id: &str, size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let path = PathBuf::from(format!("/tmp/tant2_shm_{}", session_id));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        file.set_len(size as u64)?;
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        Ok(Self {
            path,
            file,
            mmap,
            size,
        })
    }

    pub fn path(&self) -> &str {
        self.path.to_str().unwrap()
    }

    pub fn read_slice(&self, offset: usize, length: usize) -> &[u8] {
        &self.mmap[offset..offset + length]
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.size
    }
}

unsafe impl Send for ShmBuffer {}
unsafe impl Sync for ShmBuffer {}

impl Drop for ShmBuffer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
