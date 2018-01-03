use std::error::Error;

extern crate clap;
use clap::{App, AppSettings};

extern crate easage as lib;

#[macro_use] extern crate failure;

mod easage_unpack;
use easage_unpack as unpack;

mod easage_list;
use easage_list as list;

mod easage_pack;
use easage_pack as pack;

const VERSION: &'static str = "0.0.3";

#[derive(Debug, Fail)]
pub enum CliError {
    #[fail(display = "Failed to pack the given directory: {}", message)]
    PackArchive {
        message: String,
    },

    #[fail(display = "Encountered an I/O error: {}", message)]
    GeneralIoError {
        message: String,
    },

    #[fail(display = "{}", message)]
    Custom {
        message: String,
    },
}

impl From<lib::LibError> for CliError {
    fn from(e: lib::LibError) -> Self {
        CliError::Custom { message: format!("{}", e) }
    }
}

impl From<::std::io::Error> for CliError {
    fn from(e: ::std::io::Error) -> Self {
        CliError::GeneralIoError { message: e.description().to_string() }
    }
}

pub type CliResult<T> = Result<T, CliError>;

fn main() {
    let matches = App::new("easage")
        .version(VERSION)
        .about("Read, create, and unpack from BIG archives")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(list::get_command())
        .subcommand(pack::get_command())
        .subcommand(unpack::get_command())
        .get_matches();

    let run_result = match matches.subcommand() {
        (list::COMMAND_NAME, Some(args)) => list::run(args),
        (pack::COMMAND_NAME, Some(args)) => pack::run(args),
        (unpack::COMMAND_NAME, Some(args)) => unpack::run(args),
        _ => Ok(()),
    };

    if let Err(err) = run_result {
        eprintln!("ERROR: {}", err);
        std::process::exit(1);
    }
}