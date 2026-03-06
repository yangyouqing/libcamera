use core_types::{CamError, CommResult};
use core_interfaces::FileSystem;
use std::fs;
use std::io::{Read, Write};

pub struct LinuxFileSystem;

impl LinuxFileSystem {
    pub fn new() -> Self {
        Self
    }
}

impl FileSystem for LinuxFileSystem {
    fn read_file(&self, path: &str, buf: &mut [u8]) -> CommResult<usize> {
        let mut file = fs::File::open(path).map_err(|_| CamError::IoError)?;
        let n = file.read(buf).map_err(|_| CamError::IoError)?;
        Ok(n)
    }

    fn write_file(&self, path: &str, data: &[u8]) -> CommResult<()> {
        let mut file = fs::File::create(path).map_err(|_| CamError::IoError)?;
        file.write_all(data).map_err(|_| CamError::IoError)?;
        Ok(())
    }

    fn file_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    fn remove_file(&self, path: &str) -> CommResult<()> {
        fs::remove_file(path).map_err(|_| CamError::IoError)
    }

    fn file_size(&self, path: &str) -> CommResult<u64> {
        let meta = fs::metadata(path).map_err(|_| CamError::IoError)?;
        Ok(meta.len())
    }

    fn list_dir(&self, path: &str, entries: &mut [u8], _max_entries: usize) -> CommResult<usize> {
        let mut offset = 0usize;
        let dir = fs::read_dir(path).map_err(|_| CamError::IoError)?;
        for entry in dir.flatten() {
            let name = entry.file_name();
            let name_bytes = name.as_encoded_bytes();
            let needed = name_bytes.len() + 1; // name + null terminator
            if offset + needed > entries.len() {
                break;
            }
            entries[offset..offset + name_bytes.len()].copy_from_slice(name_bytes);
            entries[offset + name_bytes.len()] = 0;
            offset += needed;
        }
        Ok(offset)
    }

    fn create_dir(&self, path: &str) -> CommResult<()> {
        fs::create_dir_all(path).map_err(|_| CamError::IoError)
    }

    fn free_space(&self, path: &str) -> CommResult<u64> {
        // Use statvfs via libc
        use std::ffi::CString;
        let c_path = CString::new(path).map_err(|_| CamError::InvalidParam)?;
        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
        if ret != 0 {
            return Err(CamError::IoError);
        }
        Ok(stat.f_bfree * stat.f_bsize)
    }

    fn total_space(&self, path: &str) -> CommResult<u64> {
        use std::ffi::CString;
        let c_path = CString::new(path).map_err(|_| CamError::InvalidParam)?;
        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
        if ret != 0 {
            return Err(CamError::IoError);
        }
        Ok(stat.f_blocks * stat.f_bsize)
    }
}
