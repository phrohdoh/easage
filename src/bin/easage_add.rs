use clap::{Arg, ArgMatches, App, SubCommand};

use ::lib::{self, Kind, Archive};
use ::CliResult;

pub const COMMAND_NAME: &'static str = "add";
const ARG_NAME_SOURCE: &'static str = "source";
const ARG_NAME_FILES: &'static str = "files";
const ARG_NAME_VERBOSE: &'static str = "verbose";

pub fn get_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME)
        .about("Add files to an existing BIG archive")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .arg(Arg::with_name(ARG_NAME_SOURCE)
                .index(1)
                .takes_value(true)
                .required(true)
                .help("path to the BIG to modify"))
        .arg(Arg::with_name(ARG_NAME_FILES)
                .takes_value(true)
                .multiple(true)
                .help("path(s) to the file(s) to add to the `source` archive"))
        .arg(Arg::with_name(ARG_NAME_VERBOSE)
                .long(ARG_NAME_VERBOSE)
                .help("if supplied output more information (typically only useful for developing easage itself)"))
}

pub fn run(args: &ArgMatches) -> CliResult<()> {
    let source_path = args.value_of(ARG_NAME_SOURCE).unwrap();
    let is_verbose = args.is_present(ARG_NAME_VERBOSE);

    let mut archive = Archive::from_path(path)?;

    let kind = archive.read_kind();
    if let Kind::Unknown(bytes) = kind {
        eprintln!("Unknown archive type {:?}. Aborting.", bytes);
        return Ok(());
    }

    let table = archive.read_entry_metadata_table()?;

    let new_archive = Archive::
}
