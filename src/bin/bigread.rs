extern crate easage;

use std::env;

fn main() {
    if let Some(path) = env::args().nth(1) {
        let mut archive = easage::Archive::from_path(path).unwrap();

        let kind = archive.kind();
        if let easage::Kind::Unknown(bytes) = kind {
            eprintln!("Unknown archive type {:?}. Aborting.", bytes);
            return;
        }

        println!("Archive:");
        println!("  kind: {:?}", kind);
        println!("  size: {:?}", archive.size().unwrap());
        println!("  len: {:?}", archive.len().unwrap());
        println!("  data start: 0x{:x}", archive.data_start().unwrap());

        let table = archive.entry_metadata_table().unwrap();

        println!("Entries:");
        for (name, entry) in table.iter() {
            println!("  {}", name);
            println!("    offset: 0x{:x}", entry.offset);
            println!("    len: {}", entry.len);
        }
    }
}