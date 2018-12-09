extern crate carapace;
extern crate clap;

use std::process;

use clap::{App, Arg};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches = App::new("Carapace")
        .version(VERSION)
        .about("Shell written in Rust.")
        .arg(
            Arg::with_name("command")
                .short("c")
                .long("command")
                .value_name("command")
                .help("Commands read from string.")
                .takes_value(true)
                .conflicts_with("stdin"),
        ).arg(
            Arg::with_name("stdin")
                .short("s")
                .long("stdin")
                .help("Commands read from standard input.")
                .conflicts_with("comand"),
        ).arg(
            Arg::with_name("config")
                .long("config")
                .help(
                    "Load specific config file instead of default. \
                     The default config will be written to file if it doesn't exist.",
                ).value_name("config")
                .takes_value(true),
        ).get_matches();

    process::exit(carapace::repl(&matches));
}
