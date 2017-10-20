use ::std::io;
use ::std::path::PathBuf;
use clap::{Arg, ArgMatches, App, SubCommand};

use ::easage::{self, Kind};

pub const COMMAND_NAME: &'static str = "pack";
const ARG_NAME_SOURCE: &'static str = "source";
const ARG_NAME_OUTPUT: &'static str = "output";

pub fn get_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME)
        .version("0.0.1")
        .about("Recursively package a directory structure into a BIG archive")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .arg(Arg::with_name(ARG_NAME_SOURCE)
                .long(ARG_NAME_SOURCE)
                .value_name(ARG_NAME_SOURCE)
                .takes_value(true)
                .required(true)
                .help("Path to the directory to pack into a BIG archive."))
        .arg(Arg::with_name(ARG_NAME_OUTPUT)
                .long(ARG_NAME_OUTPUT)
                .value_name(ARG_NAME_OUTPUT)
                .takes_value(true)
                .required(true)
                .help("Path to the output BIG archive."))
}

pub fn run(args: &ArgMatches) -> io::Result<()> {
    let source = args.value_of(ARG_NAME_SOURCE).unwrap();
    let output = args.value_of(ARG_NAME_OUTPUT).unwrap();
    let output = PathBuf::from(output);

    easage::pack_directory(&source, &output, Kind::Big4, Some(b"easage0.0.1"))
}