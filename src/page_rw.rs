use crate::fs::PageFile;
use crate::fs::VolMan;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::sync::{LazyLock, Mutex};

#[cfg(feature = "std")]
pub static WRITES_REM: LazyLock<Mutex<usize>> = LazyLock::new(|| Mutex::new(27));

#[cfg(feature = "std")]
pub static PANICS_REM: LazyLock<Mutex<usize>> = LazyLock::new(|| Mutex::new(1));

pub const PAGE_SIZE: usize = 4096;

pub struct PageRW<V, F>
where V: VolMan, F: PageFile
{
    pub vm: V,
    pub file: F
}

impl <V, F> PageRW<V, F> where V: VolMan<F = F>, F: PageFile {
    pub fn new(vm: V, file: F) -> Self {
        Self {
            file: file,
            vm: vm
        }
    }

    pub fn read_page(&self, page_num: u32, buf: &mut [u8; PAGE_SIZE]) -> Result<usize, V::Error> {
        let offset: u32 = page_num * buf.len() as u32;
        self.vm.file_seek_from_start(&self.file, offset)?;
        return self.vm.file_read(&self.file, buf);
    }

    #[cfg(not(feature = "hw_failure_test"))]
    pub fn write_page(&self, page_num: u32, buf: &[u8; PAGE_SIZE]) -> Result<(), V::Error> {
        let offset: u32 = page_num * buf.len() as u32;
        self.vm.file_seek_from_start(&self.file, offset)?;
        return self.vm.file_write(&self.file, buf);
    }

    #[cfg(feature = "hw_failure_test")]
    pub fn write_page(&self, page_num: u32, buf: &[u8; PAGE_SIZE]) -> Result<(), V::Error> {
        let mut writes_rem = WRITES_REM.lock().unwrap();
        let mut panics_rem = PANICS_REM.lock().unwrap();

        if *writes_rem == 0 && *panics_rem > 0 {
            if *panics_rem > 0 {
                *panics_rem -= 1;
            }
            core::mem::drop(writes_rem);
            core::mem::drop(panics_rem);
            panic!("world ended man");
        }

        if *writes_rem > 0 {
            *writes_rem -= 1;
        }

        let offset: u32 = page_num * buf.len() as u32;
        self.vm.file_seek_from_start(&self.file, offset)?;
        return self.vm.file_write(&self.file, buf);
    }

    // this accounts for any incomplete transactions
    // so that's the reason it takes cur_db_page_count and it compares it with actual pages count
    // from file length
    pub fn extend_file_one_page(&self, cur_db_page_count: u32, buf: &mut [u8; PAGE_SIZE]) -> Result<u32, V::Error> {
        let page = (self.vm.file_length(&self.file)? / (PAGE_SIZE as u32)).min(cur_db_page_count);
        buf.fill(0);
        self.write_page(page, buf)?;
        Ok(page)
    }
}

