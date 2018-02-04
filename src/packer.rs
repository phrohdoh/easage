use ::std::fs::File;
use ::std::path::Path;
use ::std::io::{self, Read, Write};

use ::{Result, Error, Archive, Kind};

use walkdir::WalkDir;
use byteorder::{BigEndian, LittleEndian, WriteBytesExt};

pub enum EntryOrderCriteria {
    SmallestToLargest,
    Path,
}

pub struct Settings {
    pub entry_order_criteria: EntryOrderCriteria,
    pub strip_prefix: Option<String>,
    pub kind: Kind,
}

/// Recursively walk a given directory and pack *all* files into an `Archive`.
pub fn pack_directory<P>(directory: P, settings: Settings) -> Result<Archive>
    where P: AsRef<Path> {
    let directory = directory.as_ref();
    let mut entries: Vec<(String, Vec<u8>)> = vec![];

    for fs_item in WalkDir::new(directory) {
        let fs_item = fs_item?;
        let md = fs_item.metadata()?;
        if md.is_dir() {
            continue;
        }

        let path = fs_item.path();
        let source_path = path.to_path_buf();
        let mut name = source_path.to_string_lossy().to_string();

        if let Some(ref strip_prefix) = settings.strip_prefix {
            name = name.trim_left_matches(strip_prefix).to_string();
        }

        let mut f = File::open(source_path)?;
        let mut buf = Vec::with_capacity(md.len() as usize);
        let _len_read = f.read_to_end(&mut buf)?;

        entries.push((name, buf));
    }

    match settings.entry_order_criteria {
        EntryOrderCriteria::SmallestToLargest => entries.sort_by(|a, b| a.1.len().cmp(&b.1.len())),
        EntryOrderCriteria::Path => entries.sort_by(|a, b| a.0.cmp(&b.0)),
    };

    let entries = entries
        .iter()
        .map(|&(ref name, ref data)| (name.clone(), data.as_slice())) // TODO: Do this without `clone`.
        .collect::<Vec<_>>();

    let archive = pack(entries, settings.kind)?;
    Ok(archive)
}

/// Pack the given tuples of `(name, data)` into an `Archive`.
///
/// The `name` / `.0`th item in `entries` *is not* the path on disk.
/// It is the name that the given entry will have in the output archive.
pub fn pack(entries: Vec<(String, &[u8])>, kind: Kind) -> Result<Archive> {
    if entries.is_empty() {
        return Err(Error::AttemptCreateEmpty);
    }

    let table_size = entries.iter().map(|itm| {
        ::std::mem::size_of::<u32>() + // offset
        ::std::mem::size_of::<u32>() + // length
        itm.0.len() + 1 // name + null
    }).sum::<usize>();

    // NOTE: For some reason FinalBig's `data_start` is 1 byte less than ours.
    let data_start = (Archive::HEADER_LEN as usize) + table_size;
    let total_size_of_entries = entries.iter().map(|itm| itm.1.len()).sum::<usize>();
    let total_archive_size = data_start + total_size_of_entries;

    let kind_bytes = match kind {
        ::Kind::Big4 => b"BIG4",
        ::Kind::BigF => b"BIGF",
    };

    let mut buf = Vec::with_capacity(total_archive_size);

    // Write the header
    let _ = buf.write(kind_bytes)?;
    buf.write_u32::<LittleEndian>(total_archive_size as u32)?;
    buf.write_u32::<BigEndian>(entries.len() as u32)?;
    buf.write_u32::<BigEndian>(data_start as u32)?;

    // Write the entry metadata table
    let mut last_offset = data_start;
    let mut last_len = 0usize;

    for entry in &entries {
        let len = entry.1.len();
        let offset = last_offset + last_len;

        let name_bytes = entry.0.as_bytes();

        buf.write_u32::<BigEndian>(offset as u32)?;
        buf.write_u32::<BigEndian>(len as u32)?;
        let _ = buf.write(name_bytes)?;
        let _ = buf.write(&[b'\0'])?;

        last_offset = offset;
        last_len = len;
    }

    // Write the actual data
    for mut entry in entries {
        io::copy(&mut entry.1, &mut buf)?;
    }

    let ret = Archive::from_bytes(&buf)?;
    Ok(ret)
}