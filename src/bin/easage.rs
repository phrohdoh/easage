extern crate clap;
use clap::{Arg, App, SubCommand};

extern crate easage;

mod easage_extract;
use easage_extract as extract;

mod easage_list;
use easage_list as list;

fn main() {
    let matches = App::new("easage")
        .version("0.0.1")
        .about("Read, create, and extract from BIG archives")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .subcommand(extract::get_command())
        .subcommand(list::get_command())
        .get_matches();

    match matches.subcommand() {
        (extract::COMMAND_NAME, Some(args)) => extract::run(args),
        (list::COMMAND_NAME, Some(args)) => list::run(args),
        _ => Ok(()),
    };
}