use crate::{as_ref};
use crate::fs::{DbDir, Mode, PageFile, VolMan};
use crate::page_buf::{PageBuffer};
use crate::page_rw::{PageRW, PAGE_SIZE};
use crate::db::{Error, FixedPages};
use allocator_api2::alloc::Allocator;
use allocator_api2::vec::Vec;

#[cfg(feature = "std")]
extern crate std;

pub const WAL_FILE_NAME: &'static str = "DB_WAL";
pub const DB_FILE_NAME: &'static str = "DB";
pub const WAL_MAGIC: [u8; 8] = *b"WAL_FILE";
pub const WAL_TRAILER: [u8; 12] = *b"WAL_FILE_END";

#[derive(Debug)]
#[repr(C, packed)]
pub struct WalHeader {
    magic: [u8; 8],
    page_size: u32,
    page_count: u32,
}

pub struct FileHandler<V: VolMan, D: DbDir, F: PageFile> {
    cur_header: Option<WalHeader>,
    wal_file: Option<F>,
    db_dir: D,
    pub page_rw: Option<PageRW<V, F>>
}

impl WalHeader {
    fn default() -> Self {
        Self {
            magic: WAL_MAGIC,
            page_size: PAGE_SIZE as u32,
            page_count: 0
        }
    }
}

impl <V, D, F> FileHandler<V, D, F>
where
    V: VolMan<F = F, D = D>,
    D: DbDir,
    F: PageFile,
{
    pub fn new_init<A: Allocator + Clone>(
        vm: V,
        db_dir: D,
        buf: &mut PageBuffer<A>
    ) -> Result<Self, Error<V::Error>> {
        let mut fm = Self {
            wal_file: None,
            cur_header: None,
            db_dir: db_dir,
            page_rw: None
        };

        let db_file = vm.open_file_in_dir(&fm.db_dir, DB_FILE_NAME, Mode::ReadWriteCreateOrAppend)?;
        let wal_file = vm.open_file_in_dir(&fm.db_dir, WAL_FILE_NAME, Mode::ReadWriteCreateOrAppend)?;
        fm.page_rw = Some(PageRW::new(vm, db_file));
        fm.wal_file = Some(wal_file);
        match fm.wal_check_restore(buf) {
            Err(Error::InvalidWalFile) => (),
            Err(other) => return Err(other),
            Ok(_) => ()
        };

        Ok(fm)
    }

    pub fn close(&mut self) -> Result<(), Error<V::Error>> {
        if let Some(page_rw) = self.page_rw.take() {
            if let Some(f) = self.wal_file.take() {
                page_rw.vm.file_close(f)?;
            }
            page_rw.vm.delete_file_in_dir(&self.db_dir, WAL_FILE_NAME)?;
            page_rw.vm.close_dir(&self.db_dir)?;
            page_rw.vm.file_close(page_rw.file)?;
        }
        Ok(())
    }

    fn wal_check_restore<A: Allocator + Clone>(
        &mut self,
        buf: &mut PageBuffer<A>
    ) -> Result<(), Error<V::Error>> {
        {
            let wal_header = self.wal_read_header(buf)?;
            let is_magic = wal_header.magic == WAL_MAGIC;
            if !is_magic || !self.wal_verify_trailer()? {
                self.cur_header = None;
                return Ok(());
            }
        }

        let wal_header = self.wal_read_header(buf)?;

        if wal_header.page_size as usize != PAGE_SIZE {
            return Err(Error::WalNotSupported);
        }

        let page_count = wal_header.page_count;
        let page_rw = self.page_rw.as_ref().ok_or(Error::InitError)?;

        for _ in 0..page_count {
            let page = self.wal_read_u32()?;
            self.wal_read_buf(buf)?;
            page_rw.write_page(page, buf.as_mut())?;
        }

        self.cur_header = Some(WalHeader::default());
        self.wal_write_header_to_file()?;

        Ok(())
    }

    fn wal_read_u32(&self) -> Result<u32, Error<V::Error>> {
        let mut buf = [0u8; 4];
        let page_rw = self.page_rw.as_ref().ok_or(Error::InitError)?;
        let wal_file = self.wal_file.as_ref().ok_or(Error::InitError)?;
        page_rw.vm.file_read(wal_file, &mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn wal_read_buf<A: Allocator + Clone>(
        &self,
        buf: &mut PageBuffer<A>
    ) -> Result<usize, Error<V::Error>> {
        let page_rw = self.page_rw.as_ref().ok_or(Error::InitError)?;
        let wal_file = self.wal_file.as_ref().ok_or(Error::InitError)?;
        Ok(page_rw.vm.file_read(wal_file, buf.as_mut())?)
    }

    pub fn wal_read_header<A: Allocator + Clone>(
        &mut self,
        buf: &mut PageBuffer<A>,
    ) -> Result<&WalHeader, Error<V::Error>> {
        let page_rw = self.page_rw.as_ref().ok_or(Error::InitError)?;
        let wal_file = self.wal_file.as_ref().ok_or(Error::InitError)?;
        page_rw.vm.file_seek_from_start(wal_file, 0)?;
        let _ = page_rw.vm.file_read(wal_file, &mut buf.as_mut()[0..core::mem::size_of::<WalHeader>()])?;
        Ok(unsafe { as_ref!(buf, WalHeader) })
    }

    pub fn wal_verify_trailer(&mut self) -> Result<bool, Error<V::Error>> {
        let mut trailer_buf: [u8; WAL_TRAILER.len()] = [0; WAL_TRAILER.len()];
        let page_rw = self.page_rw.as_ref().ok_or(Error::InitError)?;
        let wal_file = self.wal_file.as_ref().ok_or(Error::InitError)?;
        match page_rw.vm.file_seek_from_end(wal_file, WAL_TRAILER.len() as u32) {
            Ok(_) => (),
            Err(_) => return Ok(false)
        };
        page_rw.vm.file_read(wal_file, &mut trailer_buf)?;
        Ok(trailer_buf == WAL_TRAILER)
    }

    fn wal_write_header_to_file(&mut self) -> Result<(), Error<V::Error>> {
        let page_rw = self.page_rw.as_ref().ok_or(Error::InitError)?;
        let wal_file = self.wal_file.as_ref().ok_or(Error::InitError)?;
        let header = self.cur_header.as_mut().unwrap();
        page_rw.vm.file_seek_from_start(wal_file, 0)?;
        page_rw.vm.file_write(wal_file, &header.magic)?;
        page_rw.vm.file_write(wal_file, &header.page_size.to_le_bytes())?;
        page_rw.vm.file_write(wal_file, &header.page_count.to_le_bytes())?;

        Ok(())
    }

    #[allow(unused)]
    fn wal_read_write_page_to_file<A: Allocator + Clone>(
        &mut self,
        page: u32,
        buf: &mut PageBuffer<A>
    ) -> Result<(), Error<V::Error>> {
        let wal_file = self.wal_file.as_ref().ok_or(Error::InitError)?;
        let header = self.cur_header.as_mut().unwrap();
        let page_rw = self.page_rw.as_ref().ok_or(Error::InitError)?;
        let _ = page_rw.read_page(page, buf.as_mut())?;
        page_rw.vm.file_write(wal_file, &page.to_le_bytes())?;
        page_rw.vm.file_write(wal_file, buf.as_ref())?;
        header.page_count += 1;
        Ok(())
    }

    pub fn wal_begin_write<A: Allocator + Clone>(
        &mut self,
        buf: &mut PageBuffer<A>
    ) -> Result<(), Error<V::Error>> {
        self.cur_header = Some(WalHeader::default());
        self.wal_write_header_to_file()?;
        self.wal_read_write_page_to_file(FixedPages::Header as u32, buf)?;
        self.wal_read_write_page_to_file(FixedPages::FreeList as u32, buf)?;
        self.wal_read_write_page_to_file(FixedPages::DbCat as u32, buf)?;

        Ok(())
    }

    pub fn wal_append_pages_vec<A: Allocator + Clone>(
        &mut self,
        pages: &Vec<u32, A>,
        buf: &mut PageBuffer<A>
    ) -> Result<(), Error<V::Error>> {
        for page in pages.iter() {
            self.wal_read_write_page_to_file(*page, buf)?;
        }
        Ok(())
    }

    pub fn wal_append_page<A: Allocator + Clone>(
        &mut self,
        page: u32,
        buf: &mut PageBuffer<A>
    ) -> Result<(), Error<V::Error>> {
        self.wal_read_write_page_to_file(page, buf)?;
        Ok(())
    }

    pub fn wal_write_trailer_to_file(&self) -> Result<(), Error<V::Error>> {
        let page_rw = self.page_rw.as_ref().ok_or(Error::InitError)?;
        let f = self.wal_file.as_ref().unwrap();
        page_rw.vm.file_seek_from_end(f, 0)?;
        page_rw.vm.file_write(f, &WAL_TRAILER)?;
        Ok(())
    }

    pub fn wal_end_write(&mut self) -> Result<(), Error<V::Error>> {
        self.wal_write_header_to_file()?;
        self.wal_write_trailer_to_file()?;
        let page_rw = self.page_rw.as_ref().ok_or(Error::InitError)?;
        let wal_file = self.wal_file.as_ref().ok_or(Error::InitError)?;
        page_rw.vm.file_flush(wal_file)?;
        Ok(())
    }

    pub fn end_wal(&mut self) -> Result<(), Error<V::Error>> {
        self.cur_header = Some(WalHeader::default());
        self.wal_write_header_to_file()?;
        Ok(())
    }
}

