extern crate memmap;
use memmap::{Mmap, Protection};

extern crate owning_ref;
use owning_ref::ArcRef;

extern crate byteorder;
use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

use std::io::BufRead;
use std::path::Path;
use std::sync::Arc;
use std::ops::Deref;

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
    const HEADER_LEN: u64 = 16;

    pub fn from_path<P: AsRef<Path>>(path: P) -> std::io::Result<Archive> {
        let path = path.as_ref();
        let mmap = Arc::new(Mmap::open_path(path, Protection::Read)?);

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

    pub fn size(&self) -> u32 {
        let mut values = &self[4..8];
        // TODO: Proper error handling.
        values.read_u32::<LittleEndian>().unwrap()
    }

    pub fn len(&self) -> u32 {
        let mut values = &self[8..12];
        // TODO: Proper error handling.
        values.read_u32::<BigEndian>().unwrap()
    }

    pub fn data_start(&self) -> u32 {
        let mut values = &self[12..16];
        // TODO: Proper error handling.
        values.read_u32::<BigEndian>().unwrap()
    }

    pub fn entry_metadata_table(&mut self) -> Vec<Entry> {
        let len = self.len();
        let mut c = std::io::Cursor::new(&self[..]);
        c.set_position(Self::HEADER_LEN);

        (0..len).map(|_| {
            // TODO: Proper error handling.
            let offset = c.read_u32::<BigEndian>().unwrap();
            // TODO: Proper error handling.
            let len = c.read_u32::<BigEndian>().unwrap();
            let name = {
                let mut buf = Vec::new();
                // TODO: Proper error handling.
                let name_len = c.read_until(b'\0', &mut buf).expect("Failed to read entry name");
                let name_cow = String::from_utf8_lossy(&buf[..name_len-1]);
                name_cow.to_string()
            };

            Entry {
                offset,
                len,
                name,
            }
        }).collect::<Vec<_>>()
    }
}

impl Deref for Archive {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.data
    }
}