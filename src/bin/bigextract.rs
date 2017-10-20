use std::fs::{self, OpenOptions};
use std::io::Write;

extern crate easage;

extern crate clap;
use clap::{App, Arg};

fn main() {
    let matches = App::new("bigextract")
        .version("0.0.1")
        .about("Extract the contents of a BIG archive")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .arg(Arg::with_name("source")
                .long("source")
                .value_name("SOURCE")
                .takes_value(true)
                .required(true)
                .help("Path to the BIG to unpack."))
        .arg(Arg::with_name("output")
                .long("output")
                .value_name("OUTPUT")
                .takes_value(true)
                .required(true)
                .help("Path to the directory that should contain the BIG archive's contents."))
        .get_matches();

    let source = matches.value_of("source").unwrap();
    let output = matches.value_of("output").unwrap();
    let output = std::path::PathBuf::from(output);

    let mut archive = easage::Archive::from_path(source).expect("Failed to read archive");
    let table = archive.entry_metadata_table().expect("Failed to read metadata table");
    let keys = table.keys().collect::<Vec<_>>();

    for entry_name in keys {
        if let Some(data) = archive.read_entry_by_name(entry_name) {
            let output_file = {
                let mut o = output.clone();
                o.push(entry_name);
                o
            };

            let output_dir = output_file.parent().unwrap();

            fs::create_dir_all(&output_dir);

            let mut f = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(&output_file)
                .expect(&format!("Failed to open [{}] for writing", output_file.display()));

            f.write(data).expect(&format!("Failed to write entry [{}]", output_file.display()));
        }
    }
}