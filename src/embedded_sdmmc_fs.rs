use crate::fs::{Mode, DbDir, PageFile, VolMan};
use embedded_sdmmc::{BlockDevice, TimeSource, Mode as SdMode, RawDirectory, RawFile, VolumeManager};

#[derive(Debug)]
pub struct FileSdmmc {
    pub raw: RawFile
}

#[derive(Debug)]
pub struct DbDirSdmmc {
    pub raw: RawDirectory
}

impl FileSdmmc {
    pub fn new(f: RawFile) -> Self {
        Self {
            raw: f
        }
    }
}

impl DbDirSdmmc {
    pub fn new(d: RawDirectory) -> Self {
        Self {
            raw: d
        }
    }
}

impl DbDir for DbDirSdmmc {}
impl PageFile for FileSdmmc {}


pub struct VM<
    'a, D, T,
    const MD: usize,
    const MF: usize,
    const MV: usize,
>
where
    D: BlockDevice,
    T: TimeSource
{
    vm: &'a VolumeManager<D, T, MD, MF, MV>
}

impl <
    'a, D, T,
    const MD: usize,
    const MF: usize,
    const MV: usize,
> VM<'a, D, T, MD, MF, MV>
where
    D: BlockDevice,
    T: TimeSource,
{
    pub fn new(vm: &'a VolumeManager<D, T, MD, MF, MV>) -> Self {
        Self { vm }
    }
}

impl <
    'a, D, T,
    const MD: usize,
    const MF: usize,
    const MV: usize,
> VolMan for VM<'a, D, T, MD, MF, MV>
where
    D: BlockDevice,
    T: TimeSource,
{
    type Error = embedded_sdmmc::Error<D::Error>;
    type F = FileSdmmc;
    type D = DbDirSdmmc;

    fn file_seek_from_start(&self, file: &Self::F, offset: u32) -> Result<(), Self::Error> {
        self.vm.file_seek_from_start(file.raw, offset)
    }

    fn file_seek_from_end(&self, file: &Self::F, offset: u32) -> Result<(), Self::Error> {
        self.vm.file_seek_from_end(file.raw, offset)
    }

    fn file_read(&self, file: &Self::F, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.vm.read(file.raw, buf)
    }

    fn file_write(&self, file: &Self::F, buf: &[u8]) -> Result<(), Self::Error> {
        self.vm.write(file.raw, buf)
    }

    fn file_offset(&self, file: &Self::F) -> Result<u32, Self::Error> {
        self.vm.file_offset(file.raw)
    }

    fn file_length(&self, file: &Self::F) -> Result<u32, Self::Error> {
        self.vm.file_length(file.raw)
    }

    fn file_close(&self, file: Self::F) -> Result<(), Self::Error> {
        self.vm.close_file(file.raw)
    }

    fn file_flush(&self, file: &Self::F) -> Result<(), Self::Error> {
        self.vm.flush_file(file.raw)
    }

    fn open_file_in_dir(&self, dir: &Self::D, name: &'static str, mode: Mode) -> Result<Self::F, Self::Error> {
        Ok(FileSdmmc::new(
            self.vm.open_file_in_dir(dir.raw, name, map_mode(mode))?
        ))
    }

    fn delete_file_in_dir(&self, dir: &Self::D, name: &'static str) -> Result<(), Self::Error> {
        self.vm.delete_file_in_dir(dir.raw, name)
    }
}

fn map_mode(m: Mode) -> SdMode {
    match m {
        Mode::ReadOnly => SdMode::ReadOnly,
        Mode::ReadWriteAppend => SdMode::ReadWriteAppend,
        Mode::ReadWriteTruncate => SdMode::ReadWriteTruncate,
        Mode::ReadWriteCreate => SdMode::ReadWriteCreate,
        Mode::ReadWriteCreateOrTruncate => SdMode::ReadWriteCreateOrTruncate,
        Mode::ReadWriteCreateOrAppend => SdMode::ReadWriteCreateOrAppend,
    }
}

