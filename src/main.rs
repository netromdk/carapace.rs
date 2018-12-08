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
        ).get_matches();
    process::exit(carapace::repl(&matches));
}
