use clap::{Arg, ArgMatches, App, SubCommand};

use ::lib::{Kind, Archive};
use ::CliResult;

pub const COMMAND_NAME: &'static str = "list";
const ARG_NAME: &'static str = "source";

const VERSION: &'static str = "0.0.1";

pub fn get_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME)
        .version(VERSION)
        .about("List the contents of a BIG archive")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .arg(Arg::with_name(ARG_NAME)
                .index(1)
                .takes_value(true)
                .required(true)
                .help("Path to the BIG to read."))
}

pub fn run(args: &ArgMatches) -> CliResult<()> {
    let path = args.value_of(ARG_NAME).unwrap();
    let mut archive = Archive::from_path(path)?;

    let kind = archive.read_kind();
    if let Kind::Unknown(bytes) = kind {
        eprintln!("Unknown archive type {:?}. Aborting.", bytes);
        return Ok(());
    }

    println!("Archive:");
    println!("  kind: {:?}", kind);
    println!("  size: {:?}", archive.read_size()?);
    println!("  len: {:?}", archive.read_len()?);

    let table = archive.read_entry_metadata_table()?;

    if let Some(data) = archive.read_secret_data(&table)? {
        if let Ok(s) = ::std::str::from_utf8(data) {
            println!("  secret data: {:#?}", s);
        }

        println!("  secret data len: {}", data.len());
    }

    println!("  data start: 0x{:x}", archive.read_data_start()?);

    let mut entry_info = table.iter()
        .map(|(name, entry)| (name, entry.offset, entry.len))
        .collect::<Vec<_>>();

    entry_info.sort_by(|e1, e2| (*e1.0).cmp(&e2.0));

    println!("Entries:");
    for entry in entry_info {
        println!("  {}", entry.0);
        println!("    offset: 0x{:x}", entry.1);
        println!("    len: {}", entry.2);
    }

    Ok(())
}