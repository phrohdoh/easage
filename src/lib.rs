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
//! the all-encompassing `Error` enum.
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
//! Read an archive from bytes in-memory:
//!
//! ```rust
//! use easage::{Archive, Kind};
//!
//! // This is just an example and these 4 bytes alone do not constitute a valid BIG archive.
//! let bytes = b"BIGF".to_vec();
//!
//! let archive = match Archive::from_bytes(bytes) {
//!     Ok(archive) => archive,
//!     Err(e) => {
//!         eprintln!("{}", e);
//!         return;
//!     },
//! };
//!
//! let kind = archive.read_kind().unwrap();
//! assert_eq!(kind, Kind::BigF);
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
//! let settings = Settings {
//!     // Order the archive entries alphanumeric by filepath.
//!     entry_order_criteria: EntryOrderCriteria::Path,
//!
//!     // We do not want to strip any prefix in this example.
//!     strip_prefix: None,
//!
//!     kind: Kind::BigF,
//! };
//!
//! // Finally we can create our package!
//! if let Err(e) = packer::pack_directory(directory_to_pack, &mut buf, settings) {
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
use memmap::{Mmap, MmapOptions};

extern crate owning_ref;
use owning_ref::ArcRef;

extern crate walkdir;

#[macro_use(Fail)] extern crate failure;

use std::collections::HashMap;
use std::io::{self, BufRead, Seek,  SeekFrom};
use std::ops::Deref;
use std::path::Path;
use std::fs::File;
use std::sync::Arc;

pub mod packer;

mod error;
pub use error::{Result, Error};

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO {
            inner: e,
        }
    }
}

impl From<walkdir::Error> for Error {
    fn from(e: walkdir::Error) -> Self {
        let path = e.path()
            .map(|ref_path| ref_path.to_string_lossy().to_string())
            .unwrap_or(String::from("<unknown path>"));

        Error::PathNotFound { path }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Big4,
    BigF,
}

impl Kind {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        match bytes {
            b"BIG4" => Ok(Kind::Big4),
            b"BIGF" => Ok(Kind::BigF),
            _ => Err(Error::InvalidMagic { bytes: bytes.to_vec() }),
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
#[derive(Debug, PartialEq)]
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
    /// This does not perform any data reads and as such performs no archive validation.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Archive> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let mmap = Arc::new(mmap);
        let data = ArcRef::new(mmap).map(|mm| mm.as_ref());

        Ok(Archive { data })
    }

    /// Create an anonymous memory-map and initialize an Archive structure.
    ///
    /// This does not perform any data reads and as such performs no archive validation.
    ///
    /// # Errors
    ///
    /// * If `bytes.len() == 0` this will return `Err(Error::IO)`
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Archive> {
        let mut mmap_opts = MmapOptions::new();
        let mut mmap = mmap_opts.len(bytes.len()).map_anon()?;
        mmap.copy_from_slice(&bytes);
        let mmap = mmap.make_read_only()?;
        let mmap = Arc::new(mmap);

        let data = ArcRef::new(mmap).map(|mm| mm.as_ref());
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
    /// Little-endian ASCII sequence from offset 0 to 4 (high exclusive).
    pub fn read_kind(&self) -> Result<Kind> {
        Kind::try_from_bytes(&self[0..4])
    }

    /// This is the size, in bytes, of the entire archive.
    ///
    /// Little-endian u32 from offset 4 to 8 (high exclusive).
    pub fn read_size(&self) -> Result<u32> {
        let mut values = &self[4..8];
        Ok(values.read_u32::<LittleEndian>()?)
    }

    /// This is the number of entries stored in the archive.
    ///
    /// Big-endian u32 from offset 8 to 12 (high exclusive).
    pub fn read_len(&self) -> Result<u32> {
        let mut values = &self[8..12];
        Ok(values.read_u32::<BigEndian>()?)
    }

    /// Offset at which the first entry's data starts.
    ///
    /// Big-endian u32 from offset 12 to 16 (high exclusive).
    pub fn read_data_start(&self) -> Result<u32> {
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
    pub fn read_secret_data(&mut self, table: &EntryInfoTable) -> Result<Option<&[u8]>> {
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
    pub fn read_entry_metadata_table(&mut self) -> Result<EntryInfoTable> {
        let len = self.read_len()?;

        let mut c = io::Cursor::new(&self[..]);
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_try_from_bytes_bigf() {
        let bytes = b"BIGF".to_vec();
        let kind = Kind::try_from_bytes(&bytes).unwrap();
        assert_eq!(kind, Kind::BigF);
    }

    #[test]
    fn kind_try_from_bytes_big4() {
        let bytes = b"BIG4".to_vec();
        let kind = Kind::try_from_bytes(&bytes).unwrap();
        assert_eq!(kind, Kind::Big4);
    }

    #[test]
    fn kind_try_from_bytes_err() {
        let bytes = b"".to_vec();
        assert!(match Kind::try_from_bytes(&bytes) {
            Err(Error::InvalidMagic { bytes: b }) => b == bytes,
            _ => false,
        });

        let bytes = b"BI".to_vec();
        assert!(match Kind::try_from_bytes(&bytes) {
            Err(Error::InvalidMagic { bytes: b }) => b == bytes,
            _ => false,
        });

        let bytes = b"BIG".to_vec();
        assert!(match Kind::try_from_bytes(&bytes) {
            Err(Error::InvalidMagic { bytes: b }) => b == bytes,
            _ => false,
        });

        let bytes = b"IBG".to_vec();
        assert!(match Kind::try_from_bytes(&bytes) {
            Err(Error::InvalidMagic { bytes: b }) => b == bytes,
            _ => false,
        });

        let bytes = b"BGI".to_vec();
        assert!(match Kind::try_from_bytes(&bytes) {
            Err(Error::InvalidMagic { bytes: b }) => b == bytes,
            _ => false,
        });
    }

    #[test]
    fn archive_from_bytes() {
        let result = Archive::from_bytes(vec![0]);
        assert!(result.is_ok())
    }

    #[test]
    fn archive_from_bytes_zero_length_memmap() {
        let bytes = vec![];
        let result = Archive::from_bytes(bytes);
        let err = result.err().unwrap();

        match err {
            Error::IO { inner } => {
                use std::error::Error;
                use std::io::ErrorKind;

                #[cfg(target_os = "windows")] {
                    assert_eq!(Some(87), inner.raw_os_error());
                    assert_eq!(ErrorKind::Other, inner.kind());
                    assert_eq!("other os error", inner.description());
                }

                #[cfg(not(target_os = "windows"))] {
                    assert_eq!(None, inner.raw_os_error());
                    assert_eq!(ErrorKind::InvalidInput, inner.kind());
                    assert_eq!("memory map must have a non-zero length", inner.description());
                }
             },
            _ => assert!(false),
        };
    }

    #[test]
    #[should_panic]
    // NOTE: `read_kind` panics if `bytes.len() < 4`
    // TODO: Return an error instead of panicing.
    fn archive_read_kind_panic() {
        let bytes = vec![0];
        let archive = Archive::from_bytes(bytes).unwrap();
        archive.read_kind().is_err();
    }

    #[test]
    fn archive_read_kind_bigf() {
        let bytes = b"BIGF".to_vec();
        let archive = Archive::from_bytes(bytes).unwrap();
        let kind = archive.read_kind().unwrap();
        assert_eq!(kind, Kind::BigF);
    }

    #[test]
    fn archive_read_kind_big4() {
        let bytes = b"BIG4".to_vec();
        let archive = Archive::from_bytes(bytes).unwrap();
        let kind = archive.read_kind().unwrap();
        assert_eq!(kind, Kind::Big4);
    }

    #[test]
    fn archive_read_kind_unknown() {
        let bytes = b"    ".to_vec();
        let archive = Archive::from_bytes(bytes.clone()).unwrap();
        assert!(match archive.read_kind() {
            Err(Error::InvalidMagic { bytes: b }) => b == bytes,
            _ => false,
        });

        let bytes = b"IB4G".to_vec();
        let archive = Archive::from_bytes(bytes.clone()).unwrap();
        assert!(match archive.read_kind() {
            Err(Error::InvalidMagic { bytes: b }) => b == bytes,
            _ => false,
        });
    }

    #[test]
    fn archive_read_size_0() {
        use byteorder::WriteBytesExt;

        let expected = 0;

        let mut bytes = b"BIGF".to_vec();
        bytes.write_u32::<LittleEndian>(expected).unwrap();

        let archive = Archive::from_bytes(bytes).unwrap();
        let got = archive.read_size().unwrap();

        assert_eq!(expected, got);
    }

    #[test]
    fn archive_read_size_1() {
        use byteorder::WriteBytesExt;

        let expected = 1;

        let mut bytes = b"BIGF".to_vec();
        bytes.write_u32::<LittleEndian>(expected).unwrap();

        let archive = Archive::from_bytes(bytes).unwrap();
        let got = archive.read_size().unwrap();

        assert_eq!(expected, got);
    }

    #[test]
    fn archive_read_size_u32_max() {
        use byteorder::WriteBytesExt;

        let expected = ::std::u32::MAX;

        let mut bytes = b"BIGF".to_vec();
        bytes.write_u32::<LittleEndian>(expected).unwrap();

        let archive = Archive::from_bytes(bytes).unwrap();
        let got = archive.read_size().unwrap();

        assert_eq!(expected, got);
    }

    #[test]
    #[should_panic]
    // NOTE: `read_size` panics if `bytes.len() < 8`
    // TODO: Return an error instead of panicing.
    fn archive_read_size_panic() {
        let bytes = b"BIGF".to_vec();
        let archive = Archive::from_bytes(bytes).unwrap();
        archive.read_size().unwrap();
    }
}