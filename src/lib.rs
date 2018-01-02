extern crate byteorder;
use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

extern crate memmap;
use memmap::{Mmap, Protection};

extern crate owning_ref;
use owning_ref::ArcRef;

extern crate walkdir;

use std::collections::HashMap;
use std::io::{self, BufRead, Seek,  SeekFrom};
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

mod writer;
pub use writer::pack_directory;

#[derive(Debug, Clone)]
pub enum Kind {
    Unknown(Vec<u8>),
    Big4,
    BigF,
}

impl Kind {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        match bytes {
            b"BIG4" => Kind::Big4,
            b"BIGF" => Kind::BigF,
            _ => Kind::Unknown(Vec::from(bytes)),
        }
    }
}

type EntryTable = HashMap<String, Entry>;

#[derive(Debug)]
pub struct Entry {
    pub offset: u32,
    pub len: u32,
    pub name: String,
}

pub struct Archive {
    data: ArcRef<Mmap, [u8]>,
}

/// Functions with the `read` prefix actually perform a read from
/// the memory-mapped archive. Typically you want to call these a single
/// time and refer to the resulting values in the rest of your code.
impl Archive {
    pub const HEADER_LEN: u32 = 16;

    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Archive> {
        let path = path.as_ref();
        let mmap = Mmap::open_path(path, Protection::Read)?;
        let mmap = Arc::new(mmap);

        let data = ArcRef::new(mmap).map(|mm| unsafe { mm.as_slice() });
        Ok(Archive { data })
    }

    // TODO: Consider returning a Validity enum with Valid, Bogus{Size,Len,Count,Offset}, etc variants
    pub fn is_valid(&self) -> bool {
        // TODOs:
        // - Check file size (stat) vs `size()`
        // - Sanity check `len()`
        // - Check that `data_start() < size()`
        unimplemented!()
    }

    pub fn read_kind(&self) -> Kind {
        Kind::from_bytes(&self[0..4])
    }

    pub fn read_size(&self) -> io::Result<u32> {
        let mut values = &self[4..8];
        values.read_u32::<LittleEndian>()
    }

    pub fn read_len(&self) -> io::Result<u32> {
        let mut values = &self[8..12];
        values.read_u32::<BigEndian>()
    }

    pub fn read_data_start(&self) -> io::Result<u32> {
        let mut values = &self[12..16];
        values.read_u32::<BigEndian>()
    }

    pub fn read_secret_data(&mut self) -> io::Result<Option<&[u8]>> {
        let table = match self.entry_metadata_table() {
            Ok(t) => t,
            _ => return Ok(None)
        };

        let table_size = table.iter().map(|(_k, e)|
            (std::mem::size_of::<u32>() + // offset
             std::mem::size_of::<u32>() + // length
             e.name.len() + 1) as u32 // name + null
        ).sum::<u32>();

        let data_start = self.read_data_start()? as usize;

        let secret_data_offset = (Self::HEADER_LEN + table_size) as usize;
        if secret_data_offset == data_start {
            return Ok(None);
        }

        Ok(Some(&self[secret_data_offset..data_start]))
    }

    /// Read the metadata table that lists the entries in this archive.
    /// You will need to pass the resulting table to `get_data_from_table`
    /// to retrieve actual entry data.
    pub fn read_entry_metadata_table(&mut self) -> io::Result<EntryTable> {
        let len = self.read_len()?;

        let mut c = std::io::Cursor::new(&self[..]);
        c.seek(SeekFrom::Start(Self::HEADER_LEN as u64))?;

        let mut table = EntryTable::new();

        for _ in 0..len {
            let offset = c.read_u32::<BigEndian>()?;
            let len = c.read_u32::<BigEndian>()?;
            let name = {
                let mut buf = Vec::new();
                let name_len = c.read_until(b'\0', &mut buf)?;
                let name_cow = String::from_utf8_lossy(&buf[..name_len-1]);
                name_cow.to_string()
            };

            // TODO: Investigate K=&str so `clone()` can be avoided
            table.insert(name.clone(), Entry { offset, len, name });
        }

        Ok(table)
    }

    pub fn get_bytes_via_table(&mut self, table: &EntryTable, name: &str) -> Option<&[u8]> {
        match table.get(name) {
            Some(entry) => {
                let start = entry.offset as usize;
                let end = entry.offset as usize + entry.len as usize;
                Some(&self[start..end])
            },
            None => None,
        }
    }
}

impl Deref for Archive {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}