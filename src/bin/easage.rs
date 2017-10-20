extern crate clap;
use clap::App;

extern crate easage;

mod easage_extract;
use easage_extract as extract;

mod easage_list;
use easage_list as list;

mod easage_pack;
use easage_pack as pack;

fn main() {
    let matches = App::new("easage")
        .version("0.0.1")
        .about("Read, create, and extract from BIG archives")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .subcommand(list::get_command())
        .subcommand(pack::get_command())
        .subcommand(extract::get_command())
        .get_matches();

    match matches.subcommand() {
        (list::COMMAND_NAME, Some(args)) => list::run(args),
        (pack::COMMAND_NAME, Some(args)) => pack::run(args),
        (extract::COMMAND_NAME, Some(args)) => extract::run(args),
        _ => Ok(()),
    };
}