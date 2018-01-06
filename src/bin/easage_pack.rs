use clap::{Arg, ArgMatches, App, SubCommand};

use ::lib::{Kind, packer};
use ::{CliResult, CliError};

use ::std::fs::OpenOptions;
use ::std::io::Write;

pub const COMMAND_NAME: &'static str = "pack";
const ARG_NAME_SOURCE: &'static str = "source";
const ARG_NAME_OUTPUT: &'static str = "output";
const ARG_NAME_KIND: &'static str = "kind";
const ARG_NAME_STRIP_PREFIX: &'static str = "strip-prefix";
const ARG_NAME_ORDER: &'static str = "order";

const ARG_VALUE_ORDER_SMALLEST_TO_LARGEST: &'static str = "smallest-to-largest";
const ARG_VALUE_ORDER_PATH: &'static str = "path";

pub fn get_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME)
        .about("Recursively package a directory structure into a BIG archive")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .arg(Arg::with_name(ARG_NAME_SOURCE)
                .long(ARG_NAME_SOURCE)
                .value_name(ARG_NAME_SOURCE)
                .takes_value(true)
                .required(true)
                .help("path to the directory to pack into a BIG archive"))
        .arg(Arg::with_name(ARG_NAME_OUTPUT)
                .long(ARG_NAME_OUTPUT)
                .value_name(ARG_NAME_OUTPUT)
                .takes_value(true)
                .required(true)
                .help("path to the output BIG archive"))
        .arg(Arg::with_name(ARG_NAME_KIND)
                .long(ARG_NAME_KIND)
                .value_name(ARG_NAME_KIND)
                .takes_value(true)
                .required(true)
                .possible_values(&["BIGF", "BIG4"])
                .help("archive kind (BIGF or BIG4, case-sensitive)"))
        .arg(Arg::with_name(ARG_NAME_STRIP_PREFIX)
                .long(ARG_NAME_STRIP_PREFIX)
                .value_name(ARG_NAME_STRIP_PREFIX)
                .takes_value(true)
                .help("a prefix to strip from entry names"))
        .arg(Arg::with_name(ARG_NAME_ORDER)
                .long(ARG_NAME_ORDER)
                .value_name(ARG_NAME_ORDER)
                .takes_value(true)
                .default_value(ARG_VALUE_ORDER_PATH)
                .validator(validate_order)
                .possible_values(&[ARG_VALUE_ORDER_SMALLEST_TO_LARGEST, ARG_VALUE_ORDER_PATH])
                .help("criteria used to determine entry order in the archive"))
}

pub fn run(args: &ArgMatches) -> CliResult<()> {
    let source = args.value_of(ARG_NAME_SOURCE).unwrap();

    let output = args.value_of(ARG_NAME_OUTPUT).unwrap();

    let kind = args.value_of(ARG_NAME_KIND).unwrap();
    let kind = Kind::from_bytes(kind.as_bytes());

    let strip_prefix = args.value_of(ARG_NAME_STRIP_PREFIX)
        .map(|s| s.to_string());

    let entry_order_criteria = args.value_of(ARG_NAME_ORDER)
        .map(arg_order_to_enum)
        .unwrap();

    let settings = packer::Settings {
        entry_order_criteria,
        strip_prefix,
    };

    let mut buf = vec![];

    packer::pack_directory(&source, &mut buf, kind, settings)
        .map_err(|e| CliError::PackArchive { message: format!("{}", e) })?;

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(output)?;

    Ok(file.write_all(&buf)?)
}

fn arg_order_to_enum(input: &str) -> packer::EntryOrderCriteria {
    match input {
        ARG_VALUE_ORDER_SMALLEST_TO_LARGEST => packer::EntryOrderCriteria::SmallestToLargest,
        ARG_VALUE_ORDER_PATH => packer::EntryOrderCriteria::Path,
        _  => {
            eprintln!(r#"
Unexpected error!
Please contact an author of this tool and provide the following text:

Invalid input to 'arg_order_to_enum': {:?}
Did you validate input via 'validate_order'?
"#, input);

            ::std::process::exit(1);
        },
    }
}

fn validate_order(v: String) -> Result<(), String> {
    if v == ARG_VALUE_ORDER_SMALLEST_TO_LARGEST || v == ARG_VALUE_ORDER_PATH {
        Ok(())
    } else {
        Err(format!("{} must be one of '{}' or '{}'",
            ARG_NAME_ORDER,
            ARG_VALUE_ORDER_SMALLEST_TO_LARGEST,
            ARG_VALUE_ORDER_PATH))
    }
}