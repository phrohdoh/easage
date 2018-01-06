use ::std::fs::{self, OpenOptions};
use ::std::io::Write;
use ::std::path::PathBuf;
use clap::{Arg, ArgMatches, ArgGroup, App, SubCommand};

use ::lib::Archive;
use ::{CliResult, CliError};

pub const COMMAND_NAME: &'static str = "unpack";
const ARG_NAME_SOURCE: &'static str = "source";
const ARG_NAME_OUTPUT: &'static str = "output";
const ARG_NAME_NAMES: &'static str = "names";
const ARG_NAME_ALL: &'static str = "all";

pub fn get_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME)
        .about("Unpack the contents of a BIG archive")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .arg(Arg::with_name(ARG_NAME_SOURCE)
                .long(ARG_NAME_SOURCE)
                .value_name(ARG_NAME_SOURCE)
                .required(true)
                .help("path to the BIG archive to unpack files from"))
        .arg(Arg::with_name(ARG_NAME_OUTPUT)
                .long(ARG_NAME_OUTPUT)
                .value_name(ARG_NAME_OUTPUT)
                .takes_value(true)
                .required(true)
                .help("path to the directory to write files to"))
        .arg(Arg::with_name(ARG_NAME_NAMES)
                .long(ARG_NAME_NAMES)
                .value_name(ARG_NAME_NAMES)
                .multiple(true)
                .help("one or more entry names to extract"))
        .arg(Arg::with_name(ARG_NAME_ALL)
                .long(ARG_NAME_ALL)
                .conflicts_with(ARG_NAME_NAMES)
                .help("unpack all entries"))
        .group(ArgGroup::with_name("to-extract")
                .args(&[ARG_NAME_NAMES, ARG_NAME_ALL])
                .required(true))
}

pub fn run(args: &ArgMatches) -> CliResult<()> {
    let source = args.value_of(ARG_NAME_SOURCE).unwrap();
    let output = args.value_of(ARG_NAME_OUTPUT).unwrap();
    let output = PathBuf::from(output);

    let mut names: Option<Vec<_>> = None;
    let should_unpack_all = args.is_present(ARG_NAME_ALL);

    if !should_unpack_all {
        names = Some(args.values_of(ARG_NAME_NAMES).unwrap().collect::<Vec<_>>());
    }

    let mut archive = Archive::from_path(source)?;
    let table = archive.read_entry_metadata_table()?;

    for (entry_name, _entry) in table.iter() {
        if !should_unpack_all {
            if let Some(ref names) = names.as_ref() {
                if names.contains(&entry_name.as_str()) {
                    continue;
                }
            }
        }

        if let Some(data) = archive.get_bytes_via_table(&table, entry_name) {
            let output_file = {
                let mut o = output.clone();
                o.push(entry_name.replace("\\", "/"));
                o
            };

            let output_dir = output_file.parent()
                .ok_or(CliError::Custom {
                    message: format!("Parent directory for output file {} could not be found.", output_file.display())
                })?;

            let _ = fs::create_dir_all(&output_dir);

            let mut f = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .truncate(true)
                .open(&output_file)?;

            f.write(data)?;
        }
    }

    Ok(())
}