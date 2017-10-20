use ::std::io;
use clap::{Arg, ArgMatches, App, SubCommand};

pub const COMMAND_NAME: &'static str = "extract";

pub fn get_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME)
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
}

pub fn run(args: &ArgMatches) -> io::Result<()> {
    Ok(())
}