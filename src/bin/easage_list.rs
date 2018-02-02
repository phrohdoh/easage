use clap::{Arg, ArgMatches, App, SubCommand};

use ::lib::{Archive, Error};
use ::CliResult;

pub const COMMAND_NAME: &'static str = "list";
const ARG_NAME: &'static str = "source";
const ARG_NAME_VERBOSE: &'static str = "verbose";

pub fn get_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME)
        .about("List the contents of a BIG archive")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .arg(Arg::with_name(ARG_NAME)
                .index(1)
                .takes_value(true)
                .required(true)
                .help("path to the BIG to read"))
        .arg(Arg::with_name(ARG_NAME_VERBOSE)
                .long(ARG_NAME_VERBOSE)
                .help("if supplied output more information (typically only useful for developing easage itself)"))
}

pub fn run(args: &ArgMatches) -> CliResult<()> {
    let path = args.value_of(ARG_NAME).unwrap();
    let is_verbose = args.is_present(ARG_NAME_VERBOSE);

    let mut archive = Archive::from_path(path)?;

    let kind = match archive.read_kind() {
        Ok(kind) => kind,
        Err(Error::InvalidMagic { magic }) => {
            eprintln!("Unknown archive type {:?}. Aborting.", magic);
            return Ok(());
        },
        _ => unreachable!(),
    };

    let table = archive.read_entry_metadata_table()?;

    if is_verbose {
        println!("Archive:");
        println!("  kind: {:?}", kind);
        println!("  size: {:?}", archive.read_size()?);
        println!("  len: {:?}", archive.read_len()?);

        if let Some(data) = archive.read_secret_data(&table)? {
            if let Ok(s) = ::std::str::from_utf8(data) {
                println!("  secret data: {:#?}", s);
            }

            println!("  secret data len: {}", data.len());
        }

        println!("  data start: 0x{:x}", archive.read_data_start()?);
    }

    let mut entry_info = table.iter()
        .map(|(name, entry)| (name, entry.offset, entry.len))
        .collect::<Vec<_>>();

    entry_info.sort_by(|e1, e2| (*e1.0).cmp(&e2.0));

    if is_verbose {
        println!("Entries:");
    }

    for entry in entry_info {
        if is_verbose {
            println!("  {}", entry.0);
            println!("    offset: 0x{:x}", entry.1);
            println!("    len: {}", entry.2);
        } else {
            println!("{}", entry.0);
        }
    }

    Ok(())
}
