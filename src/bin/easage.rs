use std::io;

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

mod easage_completions;
use easage_completions as completions;

const NAME: &'static str = env!("CARGO_PKG_NAME");

#[derive(Debug, Fail)]
pub enum CliError {
    #[fail(display = "Failed to pack the given directory: {:?}", inner)]
    PackArchive {
        #[cause]
        inner: lib::Error,
    },

    #[fail(display = "I/O error: {} for path {:?}", inner, path)]
    IO {
        #[cause]
        inner: io::Error,

        path: String,
    },

    #[fail(display = "{}", message)]
    Custom {
        message: String,
    },
}

impl From<lib::Error> for CliError {
    fn from(e: lib::Error) -> Self {
        CliError::Custom { message: format!("{}", e) }
    }
}

impl From<::std::io::Error> for CliError {
    fn from(e: ::std::io::Error) -> Self {
        CliError::IO {
            inner: e,
            path: "<unknown>".into(),
        }
    }
}

pub type CliResult<T> = Result<T, CliError>;

fn build_cli<'a, 'b>() -> App<'a, 'b> {
    App::new(NAME)
        .version(env!("CARGO_PKG_VERSION"))
        .about("Read, create, and unpack from BIG archives")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(completions::get_command())
        .subcommand(list::get_command())
        .subcommand(pack::get_command())
        .subcommand(unpack::get_command())
}

fn main() {
    let matches = build_cli().get_matches();

    let run_result = match matches.subcommand() {
        (completions::COMMAND_NAME, Some(args)) => completions::run(args),
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