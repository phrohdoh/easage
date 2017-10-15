extern crate easage;

use std::env;

fn main() {
    if let Some(path) = env::args().nth(1) {
        let mut archive = easage::Archive::from_path(path).unwrap();

        println!("Archive:");
        println!("  kind: {:?}", archive.kind());
        println!("  size: {:?}", archive.size());
        println!("  len: {:?}", archive.len());
        println!("  data start: {:?}", archive.data_start());

        println!("Entries:");
        for entry in archive.entry_metadata_table().iter() {
            println!("  {}", entry.name);
            println!("    offset: {}", entry.offset);
            println!("    len: {}", entry.len);
        }
    }
}