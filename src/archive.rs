use ::std;
use std::collections::HashMap;
use std::io::{self, BufRead, Seek,  SeekFrom};
use std::ops::Deref;
use std::path::Path;
use std::fs::File;
use std::sync::Arc;

use ::byteorder::{LittleEndian, BigEndian, ReadBytesExt};
use ::memmap::{Mmap, MmapOptions};
use ::owning_ref::ArcRef;

use ::{Result, Error};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Kind {
    Big4,
    BigF,
}

impl Kind {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        match bytes {
            b"BIG4" => Ok(Kind::Big4),
            b"BIGF" => Ok(Kind::BigF),
            _ => Err(Error::InvalidMagic { magic: bytes.to_vec() }),
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
    /// * If `bytes.len() == 0` this will return `Err(Error::AttemptCreateEmpty)`
    pub fn from_bytes(bytes: &[u8]) -> Result<Archive> {
        if bytes.is_empty() {
            return Err(Error::AttemptCreateEmpty);
        }

        let mut mmap_opts = MmapOptions::new();
        let mut mmap = mmap_opts.len(bytes.len()).map_anon()?;
        mmap.copy_from_slice(bytes);
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
        c.seek(SeekFrom::Start(u64::from(Self::HEADER_LEN)))?;

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

    /// Get a slice of the binary data that makes up this archive (header, table, and file data).
    ///
    /// This is useful for writing in-memory archives to, for example, files.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        &self
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
    use ::packer;
    use byteorder::LittleEndian;

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
        assert_matches!(Kind::try_from_bytes(&bytes), Err(Error::InvalidMagic { magic: ref b }) if *b == bytes);

        let bytes = b"BI".to_vec();
        assert_matches!(Kind::try_from_bytes(&bytes), Err(Error::InvalidMagic { magic: ref b }) if *b == bytes);

        let bytes = b"BIG".to_vec();
        assert_matches!(Kind::try_from_bytes(&bytes), Err(Error::InvalidMagic { magic: ref b }) if *b == bytes);

        let bytes = b"IBG".to_vec();
        assert_matches!(Kind::try_from_bytes(&bytes), Err(Error::InvalidMagic { magic: ref b }) if *b == bytes);

        let bytes = b"BGI".to_vec();
        assert_matches!(Kind::try_from_bytes(&bytes), Err(Error::InvalidMagic { magic: ref b }) if *b == bytes);
    }

    #[test]
    fn archive_from_bytes() {
        let result = Archive::from_bytes(&vec![0]);
        assert!(result.is_ok())
    }

    #[test]
    fn archive_from_bytes_zero_length_memmap() {
        let bytes = vec![];
        let result = Archive::from_bytes(&bytes);
        let err = result.err().unwrap();

        assert_matches!(err, Error::AttemptCreateEmpty);
    }

    #[test]
    #[should_panic]
    // NOTE: `read_kind` panics if `bytes.len() < 4`
    // TODO: Return an error instead of panicing.
    fn archive_read_kind_panic() {
        let bytes = vec![0];
        let archive = Archive::from_bytes(&bytes).unwrap();
        let _err = archive.read_kind();
    }

    #[test]
    fn archive_read_kind_bigf() {
        let bytes = b"BIGF".to_vec();
        let archive = Archive::from_bytes(&bytes).unwrap();
        let kind = archive.read_kind().unwrap();
        assert_eq!(kind, Kind::BigF);
    }

    #[test]
    fn archive_read_kind_big4() {
        let bytes = b"BIG4".to_vec();
        let archive = Archive::from_bytes(&bytes).unwrap();
        let kind = archive.read_kind().unwrap();
        assert_eq!(kind, Kind::Big4);
    }

    #[test]
    fn archive_read_kind_err() {
        let bytes = b"    ".to_vec();
        let archive = Archive::from_bytes(&bytes).unwrap();
        assert_matches!(archive.read_kind(), Err(Error::InvalidMagic { magic: ref b }) if *b == bytes);

        let bytes = b"IB4G".to_vec();
        let archive = Archive::from_bytes(&bytes.clone()).unwrap();
        assert_matches!(archive.read_kind(), Err(Error::InvalidMagic { magic: ref b }) if *b == bytes);
    }

    #[test]
    fn archive_read_size_0() {
        use byteorder::WriteBytesExt;

        let expected = 0;

        let mut bytes = b"BIGF".to_vec();
        bytes.write_u32::<LittleEndian>(expected).unwrap();

        let archive = Archive::from_bytes(&bytes).unwrap();
        let got = archive.read_size().unwrap();

        assert_eq!(expected, got);
    }

    #[test]
    fn archive_read_size_1() {
        use byteorder::WriteBytesExt;

        let expected = 1;

        let mut bytes = b"BIGF".to_vec();
        bytes.write_u32::<LittleEndian>(expected).unwrap();

        let archive = Archive::from_bytes(&bytes).unwrap();
        let got = archive.read_size().unwrap();

        assert_eq!(expected, got);
    }

    #[test]
    fn archive_read_size_u32_max() {
        use byteorder::WriteBytesExt;

        let expected = ::std::u32::MAX;

        let mut bytes = b"BIGF".to_vec();
        bytes.write_u32::<LittleEndian>(expected).unwrap();

        let archive = Archive::from_bytes(&bytes).unwrap();
        let got = archive.read_size().unwrap();

        assert_eq!(expected, got);
    }

    #[test]
    #[should_panic]
    // NOTE: `read_size` panics if `bytes.len() < 8`
    // TODO: Return an error instead of panicing.
    fn archive_read_size_panic() {
        let bytes = b"BIGF";
        let archive = Archive::from_bytes(&bytes[..]).unwrap();
        archive.read_size().unwrap();
    }

    #[test]
    fn archive_read_entry_metadata_table() {
        let name1 = "first/entry.txt";
        let data1 = [0, 1, 2, 3];

        let name2 = "second/entry/bar.txt";
        let data2 = [0, 9, 8, 7];

        let entries = vec![
            (name1.into(), &data1[..]),
            (name2.into(), &data2[..]),
        ];

        let mut archive = packer::pack(entries, Kind::BigF).unwrap();
        let table = archive.read_entry_metadata_table();
        assert!(table.is_ok());
        let table = table.unwrap();

        assert!(table.contains_key(name1));
        assert!(table.contains_key(name2));
        assert!(!table.contains_key("some/other/key.ini"));
    }

    #[test]
    fn archive_get_bytes_via_table() {
        let name = "first/entry.txt";
        let data = [0, 1, 2, 3];

        let entries = vec![(name.into(), &data[..])];

        let mut archive = packer::pack(entries, Kind::BigF).unwrap();

        let table = archive.read_entry_metadata_table();
        assert!(table.is_ok());
        let table = table.unwrap();
        assert!(table.contains_key(name));

        let bytes = archive.get_bytes_via_table(&table, name);
        assert!(bytes.is_some());

        let bytes = bytes.unwrap();
        assert_eq!(data, bytes);
    }

    #[test]
    fn archive_get_bytes_via_table_empty() {
        let name = "first/entry.txt";
        let data: [u8; 0] = [];

        let entries = vec![(name.into(), &data[..])];

        let mut archive = packer::pack(entries, Kind::BigF).unwrap();

        let table = archive.read_entry_metadata_table();
        assert!(table.is_ok());
        let table = table.unwrap();
        assert!(table.contains_key(name));

        let bytes = archive.get_bytes_via_table(&table, name);
        assert!(bytes.is_some());

        let bytes = bytes.unwrap();
        assert!(bytes.is_empty());
    }
}