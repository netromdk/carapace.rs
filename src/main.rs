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
                .value_name("CMD")
                .help("Commands read from string.")
                .takes_value(true),
        ).arg(
            Arg::with_name("stdin")
                .short("s")
                .long("stdin")
                .help("Commands read from standard input."),
        ).get_matches();

    if matches.is_present("command") && matches.is_present("stdin") {
        println!("--command and --stdin cannot be used together!");
        process::exit(1);
    }

    process::exit(carapace::repl(&matches));
}
