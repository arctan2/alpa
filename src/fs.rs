pub trait PageFile {}

pub trait DbDir {}

pub enum Mode {
    ReadOnly,
    ReadWriteAppend,
    ReadWriteTruncate,
    ReadWriteCreate,
    ReadWriteCreateOrTruncate,
    ReadWriteCreateOrAppend,
}

pub trait VolMan {
    type Error: core::fmt::Debug;
    type F: PageFile;
    type D: DbDir;

    fn file_seek_from_start(&self, file: &Self::F, offset: u32) -> Result<(), Self::Error>;
    fn file_seek_from_end(&self, file: &Self::F, offset: u32) -> Result<(), Self::Error>;
    fn file_read(&self, file: &Self::F, buf: &mut [u8]) -> Result<usize, Self::Error>;
    fn file_write(&self, file: &Self::F, buf: &[u8]) -> Result<(), Self::Error>;
    fn file_offset(&self, file: &Self::F) -> Result<u32, Self::Error>;
    fn file_length(&self, file: &Self::F) -> Result<u32, Self::Error>;
    fn file_close(&self, file: Self::F) -> Result<(), Self::Error>;
    fn file_flush(&self, file: &Self::F) -> Result<(), Self::Error>;

    fn close_dir(&self, dir: &Self::D) -> Result<(), Self::Error>;
    fn open_file_in_dir(&self, dir: &Self::D, name: &'static str, mode: Mode) -> Result<Self::F, Self::Error>;
    fn delete_file_in_dir(&self, dir: &Self::D, name: &'static str) -> Result<(), Self::Error>;
}

