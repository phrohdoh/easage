use ::std::error::Error;
use ::std::fs::{File, OpenOptions};
use ::std::io::{self, BufWriter, Write};
use ::std::path::{Path, PathBuf};

use ::{LibResult, LibError};

use walkdir::WalkDir;
use byteorder::{BigEndian, LittleEndian, WriteBytesExt};

pub enum EntryOrderCriteria {
    SmallestToLargest,
    Path,
}

pub struct Settings {
    pub entry_order_criteria: EntryOrderCriteria,
    pub strip_prefix: Option<String>,
}

/// Note: If you pass `Kind::Unknown(..)` to this function it will return a `LibResult::Err(LibError::InvalidKind)`.
pub fn pack_directory<P1, P2>(input_directory: P1, output_filepath: P2, kind: ::Kind, settings: Settings) -> LibResult<()>
    where P1: AsRef<Path>,
          P2: AsRef<Path> {
    let input_directory = input_directory.as_ref();
    let output_filepath = output_filepath.as_ref();

    let mut entries = vec![];

    let mut total_size_of_entries = 0u32;
    for entry_res in WalkDir::new(input_directory) {
        let entry = entry_res?;

        let md = entry.metadata()?;
        if md.is_dir() {
            continue;
        }

        let path = entry.path();
        let len = md.len() as u32;
        total_size_of_entries += len;

        let source_path = path.to_path_buf();

        let mut output_filepath = source_path.to_string_lossy().to_string();

        if let Some(ref strip_prefix) = settings.strip_prefix {
            if output_filepath.starts_with(strip_prefix) {
                output_filepath = output_filepath.trim_left_matches(strip_prefix).to_string();
            }
        }

        entries.push(Entry::new(source_path, output_filepath, len));
    }

    if entries.len() == 0 {
        return Err(LibError::Custom { message: String::from("Found no files to pack") });
    }

    match settings.entry_order_criteria {
        EntryOrderCriteria::SmallestToLargest => entries.sort_by(|a, b| a.len.cmp(&b.len)),
        EntryOrderCriteria::Path => entries.sort_by(|a, b| a.source_path.cmp(&b.source_path)),
    };

    let table_size = calc_table_size(entries.iter());

    // NOTE: For some reason FinalBig's `data_start` is 1 byte less than ours.
    let data_start = ::Archive::HEADER_LEN + table_size;

    let kind_bytes = match kind {
        ::Kind::Big4 => "BIG4",
        ::Kind::BigF => "BIGF",
        _ => return Err(LibError::InvalidKind),
    }.as_bytes();

    let buf = Vec::with_capacity(data_start as usize);
    let mut writer = BufWriter::new(buf);

    // Write the header
    writer.write(kind_bytes)?;
    writer.write_u32::<LittleEndian>(data_start + total_size_of_entries)?;
    writer.write_u32::<BigEndian>(entries.len() as u32)?;
    writer.write_u32::<BigEndian>(data_start)?;

    // Write the entry metadata table
    let mut last_offset = data_start;
    let mut last_len = 0u32;

    for entry in &entries {
        let len = entry.len;
        let offset = last_offset + last_len;

        let path_bytes = entry.output_filepath.as_bytes();

        writer.write_u32::<BigEndian>(offset)?;
        writer.write_u32::<BigEndian>(len as u32)?;
        writer.write(path_bytes)?;
        writer.write(&[b'\0'])?;

        last_offset = offset;
        last_len = len;
    }

    // Write the actual data
    for entry in entries {
        let mut f = File::open(&entry.source_path).map_err(|_e|
            LibError::Custom { message: format!("Failed to open file {:?} for reading.", entry.source_path)
        })?;

        io::copy(&mut f, &mut writer)?;
    }

    let inner = writer.into_inner().map_err(|e| LibError::Custom { message: e.description().to_string() })?;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_filepath)?;

    file.write_all(&inner)?;

    Ok(())
}

struct Entry {
    source_path: PathBuf,
    output_filepath: String,
    len: u32,
}

impl Entry {
    fn new(source_path: PathBuf, output_filepath: String, len: u32) -> Self {
        Self { source_path, output_filepath, len }
    }
}

fn calc_table_size<'e, I: Iterator<Item=&'e Entry>>(entries: I) -> u32 {
    entries.map(table_record_size).sum()
}

fn table_record_size(e: &Entry) -> u32 {
    (::std::mem::size_of::<u32>() + // offset
     ::std::mem::size_of::<u32>() + // length
     e.output_filepath.len() + 1) as u32 // name + null
}