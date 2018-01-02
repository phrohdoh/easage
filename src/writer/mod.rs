use ::std::error::Error;
use ::std::borrow::Cow;
use ::std::fs::{File, OpenOptions};
use ::std::io::{self, BufWriter, Write};
use ::std::path::{Path, PathBuf};

use ::{LibResult, LibError};

use walkdir::WalkDir;
use byteorder::{BigEndian, LittleEndian, WriteBytesExt};

struct Entry {
    source_path: PathBuf,
    len: u32,
}

impl Entry {
    pub fn new(source_path: PathBuf, len: u32) -> Self {
        Self { source_path, len }
    }

    fn source_path_lossy(&self) -> Cow<str> {
        self.source_path.to_string_lossy()
    }
}

pub fn pack_directory<P1, P2>(input_directory: P1, output_filepath: P2, kind: ::Kind, strip_prefix: Option<&str>) -> LibResult<()>
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

        entries.push(Entry::new(path.to_path_buf(), len));
    }

    if entries.len() == 0 {
        return Err(LibError::Custom { message: String::from("Found no files to pack") });
    }

    // TODO: Expose sort order as an option.
    // The reasoning being one user may prefer alphanumeric order
    // while another may want to store from smallest to largest.
    // Example: The community-driven tool FinalBig orders by entry path.
    entries.sort_by(|a, b| a.len.cmp(&b.len));

    let table_size = calc_table_size(entries.iter());

    // NOTE: For some reason FinalBig's `data_start` is 1 byte less than ours.
    let data_start = ::Archive::HEADER_LEN + table_size;

    let kind_bytes = match kind {
        ::Kind::Big4 => "BIG4",
        ::Kind::BigF => "BIGF",
        _ => panic!("TODO: Return an error if called with Kind::Unknown")
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

        let mut path = entry.source_path_lossy().to_string();

        if let Some(strip_prefix) = strip_prefix {
            if path.starts_with(strip_prefix) {
                path = path.trim_left_matches(strip_prefix).to_string();
            }
        }

        let path_bytes = path.as_bytes();

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

fn calc_table_size<'e, I: Iterator<Item=&'e Entry>>(entries: I) -> u32 {
    entries.map(table_record_size).sum()
}

fn table_record_size(e: &Entry) -> u32 {
    (::std::mem::size_of::<u32>() + // offset
     ::std::mem::size_of::<u32>() + // length
     e.source_path_lossy().len() + 1) as u32 // name + null
}