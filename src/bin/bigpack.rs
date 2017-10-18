extern crate easage;

extern crate walkdir;
use walkdir::WalkDir;

extern crate byteorder;
use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};

extern crate clap;
use clap::{App, Arg};

use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::io::{BufWriter, Write};
use std::ffi::OsString;

fn main() {
    let matches = App::new("bigpack")
       .version("0.0.1")
       .about("Recursively package a directory structure into a BIG archive")
       .author("Taryn Hill <taryn@phrohdoh.com>")
        .arg(Arg::with_name("source")
            .long("source")
            .value_name("SOURCE")
            .takes_value(true)
            .required(true)
            .help("Path to the directory to pack into a BIG archive."))
       .arg(Arg::with_name("output")
            .long("output")
            .value_name("OUTPUT")
            .takes_value(true)
            .required(true)
            .help("Path to the output BIG archive."))
       .get_matches(); 

    let source_dir = matches.value_of("source").unwrap();
    let output = matches.value_of("output").unwrap();

    compress_dir_to_big(&source_dir, &output);
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

fn compress_dir_to_big(dir_path: &str, output_path: &str) {
    let mut dir_path = dir_path;
    dir_path = dir_path.trim_right_matches('/').trim_right_matches('\\');

    let dir_path = Path::new(dir_path);

    let mut entries = vec![];

    for entry in WalkDir::new(dir_path) {
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
    let mut writer = BufWriter::new(buf);

    let table_size = calc_table_size(entries.iter());
    let data_start = easage::Archive::HEADER_LEN + table_size;

    // Write the header
    writer.write(b"BIG4").expect("Failed to write format bytes");
    writer.write_u32::<LittleEndian>(0).expect("Failed to write [bogus] size");
    writer.write_u32::<BigEndian>(entries.len() as u32).expect("Failed to write len");
    writer.write_u32::<BigEndian>(data_start).expect("Failed to write data_start");

    let mut last_len = 0u32;
    for entry in &entries {
        let len = entry.len;
        let offset = data_start + last_len;
        let name_bytes = entry.name.to_str().unwrap().as_bytes();

        writer.write_u32::<BigEndian>(offset).expect("Failed to write entry's offset");
        writer.write_u32::<BigEndian>(len as u32).expect("Failed to write entry's len");
        writer.write(name_bytes).expect("Failed to write entry's name");
        writer.write(&[b'\0']).expect("Failed to write entry's name's leading NUL");

        last_len = len;
    }

    for entry in entries {
        let mut f = std::fs::File::open(entry.name).unwrap();
        std::io::copy(&mut f, &mut writer).expect("Failed to write entry's data");
    }

    let mut inner = writer.into_inner().unwrap();
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(output_path)
        .unwrap();

    file.write_all(&inner).expect(&format!("Failed to write {}", output_path));
}

fn calc_table_size<'e, I: Iterator<Item=&'e Entry>>(entries: I) -> u32 {
    entries.map(|e| table_record_size(e)).sum()
}

fn table_record_size(e: &Entry) -> u32 {
    (std::mem::size_of::<u32>() + // offset
     std::mem::size_of::<u32>() + // length
     e.name.to_str().unwrap().len() + 1) as u32 // name + null
}