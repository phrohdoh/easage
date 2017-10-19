extern crate easage;

extern crate clap;
use clap::{App, Arg};

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

    easage::pack_directory(&source_dir, &output, easage::Kind::Big4, Some(b"easage0.0.1"))
        .expect(&format!("Failed to pack {} into {}", source_dir, output));
}