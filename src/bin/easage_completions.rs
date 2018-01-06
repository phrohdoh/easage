use clap::{Arg, ArgMatches, App, SubCommand};
use ::CliResult;

pub const COMMAND_NAME: &'static str = "completions";
const ARG_NAME_SHELL: &'static str = "shell";

pub fn get_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(COMMAND_NAME)
        .about("Generate tab-completion scripts (prints to stdout)")
        .author("Taryn Hill <taryn@phrohdoh.com>")
        .arg(Arg::with_name(ARG_NAME_SHELL)
                .required(true)
                .possible_values(&["bash", "fish", "powershell", "zsh"])
                .help("the shell to generate the script for"))
}

pub fn run(args: &ArgMatches) -> CliResult<()> {
    let shell = args.value_of(ARG_NAME_SHELL).unwrap();
    let shell = shell.parse().unwrap();

    Ok(::build_cli().gen_completions_to(::NAME, shell, &mut ::std::io::stdout()))
}