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

#[derive(Debug)]
pub struct Entry {
    pub offset: u32,
    pub len: u32,
    pub name: String,
}

pub struct Archive {
    data: ArcRef<Mmap, [u8]>,
}

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

    pub fn kind(&self) -> Kind {
        match &self[0..4] {
            b"BIG4" => Kind::Big4,
            b"BIGF" => Kind::BigF,
            bytes => Kind::Unknown(Vec::from(bytes)),
        }
    }

    pub fn size(&self) -> io::Result<u32> {
        let mut values = &self[4..8];
        values.read_u32::<LittleEndian>()
    }

    pub fn len(&self) -> io::Result<u32> {
        let mut values = &self[8..12];
        values.read_u32::<BigEndian>()
    }

    pub fn data_start(&self) -> io::Result<u32> {
        let mut values = &self[12..16];
        values.read_u32::<BigEndian>()
    }

    pub fn entry_metadata_table(&mut self) -> io::Result<HashMap<String, Entry>> {
        let len = self.len()?;

        let mut c = std::io::Cursor::new(&self[..]);
        c.seek(SeekFrom::Start(Self::HEADER_LEN as u64))?;

        let mut table = HashMap::new();

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

    pub fn read_entry_by_name(&mut self, name: &str) -> Option<&[u8]> {
        // TODO: Bubble up error.
        let table = match self.entry_metadata_table() {
            Ok(t) => t,
            _ => return None
        };

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