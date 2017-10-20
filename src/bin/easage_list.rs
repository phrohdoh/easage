use ::std::io;
use clap::{Arg, ArgMatches, App, SubCommand};

use ::easage::{Kind, Archive};

pub const COMMAND_NAME: &'static str = "list";
const ARG_NAME: &'static str = "source";

pub fn get_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME)
        .version("0.0.1")
        .about("List the contents of a BIG archive")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .arg(Arg::with_name(ARG_NAME)
                .index(1)
                .takes_value(true)
                .required(true)
                .help("Path to the BIG to read."))
}

pub fn run(args: &ArgMatches) -> io::Result<()> {
    let path = args.value_of(ARG_NAME).unwrap();
    let mut archive = Archive::from_path(path)?;

    let kind = archive.kind();
    if let Kind::Unknown(bytes) = kind {
        eprintln!("Unknown archive type {:?}. Aborting.", bytes);
        return Ok(());
    }

    println!("Archive:");
    println!("  kind: {:?}", kind);
    println!("  size: {:?}", archive.size()?);
    println!("  len: {:?}", archive.len()?);
    println!("  data start: 0x{:x}", archive.data_start()?);

    let table = archive.entry_metadata_table()?;

    println!("Entries:");
    for (name, entry) in table.iter() {
        println!("  {}", name);
        println!("    offset: 0x{:x}", entry.offset);
        println!("    len: {}", entry.len);
    }

    Ok(())
}
