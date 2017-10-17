extern crate easage;

extern crate walkdir;
use walkdir::WalkDir;

extern crate byteorder;
use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};

use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::io::{BufWriter, Write};
use std::ffi::OsString;

fn main() {
    let mut args = env::args();
    if let Some(target_dir_path) = args.nth(1) {
        compress_dir_to_big(&target_dir_path);
    } else {
        eprintln!("Please give me a directory to compress!");
        std::process::exit(1);
    }
}

struct Entry {
    name: PathBuf,
    len: u32,
}

impl Entry {
    pub fn new(name: PathBuf, len: u32) -> Self {
        Self {
            name,
            len,
        }
    }
}

fn compress_dir_to_big(path: &str) {
    println!("Compressing [{}]", path);
    let mut path = path;
    if path.ends_with('/') {
        path = path.trim_right_matches('/');
    }

    let path = Path::new(path);

    let mut entries = vec![];

    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();

        let md = entry.metadata().unwrap();
        if md.is_dir() {
            continue;
        }

        let path = entry.path().to_path_buf();

        entries.push(Entry::new(path, md.len() as u32));
    }

    entries.sort_by(|a, b| a.len.cmp(&b.len));

    let buf = vec![];
    //let mut writer = BufWriter::new(buf);
    let mut writer = std::io::Cursor::new(buf);

    let table_size = calc_table_size(entries.iter());
    let data_start = easage::Archive::HEADER_LEN + table_size;

    // Write the header
    writer.write(b"BIG4").expect("Failed to write format bytes");
    writer.write_u32::<LittleEndian>(0).expect("Failed to write [bogus] size");
    writer.write_u32::<BigEndian>(entries.len() as u32).expect("Failed to write len");
    writer.write_u32::<BigEndian>(data_start).expect("Failed to write data_start");

    println!("Finished writing the header at {}", writer.position());

    let mut last_len = 0u32;
    for entry in &entries {
        let len = entry.len;
        let offset = data_start + last_len;
        writer.write_u32::<BigEndian>(offset).expect("Failed to write entry's offset");
        writer.write_u32::<BigEndian>(len as u32).expect("Failed to write entry's len");

        let name = entry.name.to_str().unwrap();
        println!("Writing data for {} @ {} for {} bytes", name, offset, len);
        let name_bytes = name.as_bytes();

        writer.write(name_bytes).expect("Failed to write entry's name");
        writer.write(&[b'\0']).expect("Failed to write entry's name's leading NUL");
    }

    println!("Finished writing the table at {}", writer.position());

    for entry in entries {
        let mut f = std::fs::File::open(entry.name).unwrap();
        std::io::copy(&mut f, &mut writer).expect("Failed to write entry's data");
    }

    let mut inner = writer.into_inner();
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("_test.big")
        .unwrap();

    file.write_all(&inner).expect("Failed to write _test.big");
}

fn calc_table_size<'e, I: Iterator<Item=&'e Entry>>(entries: I) -> u32 {
    entries.map(|e| table_record_size(e)).sum()
}

fn table_record_size(e: &Entry) -> u32 {
    (std::mem::size_of::<u32>() + // offset
     std::mem::size_of::<u32>() + // length
     e.name.to_str().unwrap().len() + 1) as u32 // name + null
}