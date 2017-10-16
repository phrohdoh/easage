extern crate easage;

use std::env;

fn main() {
    if let Some(path) = env::args().nth(1) {
        let mut archive = easage::Archive::from_path(path).unwrap();

        println!("Archive:");
        println!("  kind: {:?}", archive.kind());
        println!("  size: {:?}", archive.size());
        println!("  len: {:?}", archive.len());
        println!("  data start: 0x{:x}", archive.data_start());

        let table = archive.entry_metadata_table();

        println!("Entries:");
        for (name, entry) in table.iter() {
            println!("  {}", name);
            println!("    offset: 0x{:x}", entry.offset);
            println!("    len: {}", entry.len);
        }
    }
}