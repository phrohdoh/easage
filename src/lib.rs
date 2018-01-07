//! `easage` provides programmatic manipulation of BIG archives.
//!
//! BIG files are an archive format used in many games published by Electronic Arts.
//!
//! The BIG format is conceptually similar to TAR. It has magic, a header, and data.
//!
//! Neither compressed nor encrypted BIG formats are supported by easage at this time.
//!
//! # Getting started
//!
//! Read the below examples then check out the `Archive` struct.
//!
//! # Error handling
//!
//! Errors are bubbled-up via the [`failure`](https://crates.io/crates/failure) crate into
//! the all-encompassing `LibError` enum.
//!
//! All errors implement the `Display` trait so feel free to print away!
//!
//! <small>
//! Side note: I don't think I am using all the goodies `failure` has to offer.
//! Please let me know if you see something that can be improved.
//! </small>
//!
//! # Examples
//!
//! Read an archive from a file:
//!
//! ```rust,no_run
//! use easage::Archive;
//!
//! // This must be a type that implements `AsRef<Path>`.
//! let path = "path/to/your.big";
//!
//! let archive = match Archive::from_path(path) {
//!     Ok(archive) => archive,
//!     Err(e) => {
//!         eprintln!("{}", e);
//!         return;
//!     },
//! };
//! ```
//!
//! Getting data of a file that is inside of an archive:
//!
//! ```rust,no_run
//! use easage::Archive;
//!
//! let mut archive = Archive::from_path("path/to/your.big").unwrap();
//!
//! // This provides us with a lookup table so we don't
//! // have to read the header repeatedly.
//! let table = match archive.read_entry_metadata_table() {
//!     Ok(table) => table,
//!     Err(e) => {
//!         eprintln!("{}", e);
//!         return;
//!     },
//! };
//!
//! // NOTE: `table` is an easage::EntryInfoTable which
//! // you can `.iter()` over to inspect all entries.
//!
//! if let Some(data) = archive.get_bytes_via_table(&table, "your/entry/name.txt") {
//!     // data: &[u8]
//! }
//! ```
//!
//! Package a directory (recursively) into an archive:
//!
//! ```rust,no_run
//! use std::io::Write;
//! use easage::Kind;
//! use easage::packer::{self, Settings, EntryOrderCriteria};
//!
//! // This must be a type that implements `AsRef<Path>`.
//! let directory_to_pack = "path/to/a/directory";
//!
//! // Where to write the binary data to.
//! // This must be a type that implements `Write`.
//! let mut buf = vec![];
//!
//! // This is the 'magic' that identifies this file
//! // as a BIG archive. Only Big4 and BigF are supported currently.
//! let kind = Kind::BigF;
//!
//! let settings = Settings {
//!     // Order the archive entries alphanumeric by filepath.
//!     entry_order_criteria: EntryOrderCriteria::Path,
//!
//!     // We do not want to strip any prefix in this example.
//!     strip_prefix: None,
//! };
//!
//! // Finally we can create our package!
//! if let Err(e) = packer::pack_directory(directory_to_pack, &mut buf, kind, settings) {
//!     eprintln!("{}", e);
//! }
//!
//! // At this point you probably want to write `buf` to a file.
//! use std::fs::OpenOptions;
//!
//! let mut file = OpenOptions::new()
//!     .write(true)
//!     .create(true)
//!     .open("my_archive.big")
//!     .expect("Failed to open file for writing.");
//!
//! file.write_all(&buf).expect("Failed to write data to the new file.");
//! ```

extern crate byteorder;
use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

extern crate memmap;
use memmap::{Mmap, Protection};

extern crate owning_ref;
use owning_ref::ArcRef;

extern crate walkdir;

#[macro_use(Fail)] extern crate failure;

use std::error::Error;
use std::collections::HashMap;
use std::io::{BufRead, Seek,  SeekFrom};
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

pub mod packer;

#[derive(Debug, Fail)]
pub enum LibError {
    #[fail(display = "Unable to find the path '{}'. Perhaps it does not exist or you do not have the required permissions.", path)]
    PathNotFound {
        path: String,
    },

    #[fail(display = "The archive kind you gave is invalid in this context")]
    InvalidKind,

    #[fail(display = "{}", message)]
    Custom {
        message: String,
    },
}

impl From<std::io::Error> for LibError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => LibError::Custom {
                message: e.cause().map(|err| err.description().to_string())
                    .unwrap_or(String::from("A path was not found (exactly which path is currently unknown)"))
            },
            _ => LibError::Custom { message: e.description().to_string() },
        }
    }
}

impl From<walkdir::Error> for LibError {
    fn from(e: walkdir::Error) -> Self {
        let path =  e.path()
            .map(|ref_path| ref_path.to_string_lossy().to_string())
            .unwrap_or(String::from("<unknown path>"));

        LibError::PathNotFound { path }
    }
}

pub type LibResult<T> = Result<T, LibError>;

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

/// A map from entry name to metadata present in the header of an Archive.
pub type EntryInfoTable = HashMap<String, EntryInfo>;

/// Metadata that describes a single entry
/// in the owning Archive.
///
/// This struct contains none of the actual file data.
#[derive(Debug)]
pub struct EntryInfo {
    pub offset: u32,
    pub len: u32,
    pub name: String,
}

/// A file container.
///
/// Library users start here!
pub struct Archive {
    data: ArcRef<Mmap, [u8]>,
}

/// Functions with the `read_` prefix actually perform a read from
/// the memory-mapped archive.
///
/// Typically you want to call these a single time and refer to
/// the resulting values in the rest of your code.
impl Archive {
    #[doc(hidden)]
    pub const HEADER_LEN: u32 = 16;

    /// Memory-map the given filepath and initialize an Archive structure.
    ///
    /// This does not perform any data reads and as such performs no validation.
    pub fn from_path<P: AsRef<Path>>(path: P) -> LibResult<Archive> {
        let path = path.as_ref();
        let mmap = Mmap::open_path(path, Protection::Read)?;
        let mmap = Arc::new(mmap);

        let data = ArcRef::new(mmap).map(|mm| unsafe { mm.as_slice() });
        Ok(Archive { data })
    }

    // TODO: Consider returning a Validity enum with Valid, Bogus{Size,Len,Count,Offset}, etc variants
    #[doc(hidden)]
    pub fn is_valid(&self) -> bool {
        // TODOs:
        // - Check file size (stat) vs `size()`
        // - Sanity check `len()`
        // - Check that `data_start() < size()`
        unimplemented!()
    }

    /// The file signature that indicates whether or not
    /// this is a BIG archive.
    ///
    /// Little-endian ASCII sequence from offset 4 to 8 (high exclusive).
    pub fn read_kind(&self) -> Kind {
        Kind::from_bytes(&self[0..4])
    }

    /// This is the size, in bytes, of the entire archive.
    ///
    /// Little-endian u32 from offset 4 to 8 (high exclusive).
    pub fn read_size(&self) -> LibResult<u32> {
        let mut values = &self[4..8];
        Ok(values.read_u32::<LittleEndian>()?)
    }

    /// This is the number of entries stored in the archive.
    ///
    /// Big-endian u32 from offset 8 to 12 (high exclusive).
    pub fn read_len(&self) -> LibResult<u32> {
        let mut values = &self[8..12];
        Ok(values.read_u32::<BigEndian>()?)
    }

    /// Offset at which the first entry's data starts.
    ///
    /// Big-endian u32 from offset 12 to 16 (high exclusive).
    pub fn read_data_start(&self) -> LibResult<u32> {
        let mut values = &self[12..16];
        Ok(values.read_u32::<BigEndian>()?)
    }

    /// There is potentially a gap between the end of the header
    /// and the start of the data we care about. I affectionately
    /// refer to this as "secret data".
    ///
    /// You probably don't care about this.
    ///
    /// I do not know if this needs to be aligned to a particular
    /// size for other BIG-manipulating tools to read it.
    pub fn read_secret_data(&mut self, table: &EntryInfoTable) -> LibResult<Option<&[u8]>> {
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
    pub fn read_entry_metadata_table(&mut self) -> LibResult<EntryInfoTable> {
        let len = self.read_len()?;

        let mut c = std::io::Cursor::new(&self[..]);
        c.seek(SeekFrom::Start(Self::HEADER_LEN as u64))?;

        let mut table = EntryInfoTable::new();

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
            table.insert(name.clone(), EntryInfo { offset, len, name });
        }

        Ok(table)
    }

    /// Given a table from this archive's `read_entry_metadata_table` and an
    /// entry name return the data of the named file if this archive
    /// contains a file by that name.
    ///
    /// # Panics
    /// If you provide this a table from a different archive that happens to
    /// share an entry name with an entry in this archive this *may* panic.
    ///
    /// A panic will occurr if data start or end for an entry lies outside
    /// of the archive file's boundaries.
    pub fn get_bytes_via_table(&mut self, table: &EntryInfoTable, name: &str) -> Option<&[u8]> {
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

#[doc(hidden)]
impl Deref for Archive {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
