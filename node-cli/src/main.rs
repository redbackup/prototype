#[macro_use]
extern crate clap;
extern crate redbackup_node;

use clap::{App, Arg};
use redbackup_node::config::{Config, ParseError};
use std::process;

fn main() {
    let matches = App::new("redbackup node-cli")
        .about("redbackup node server")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("ip")
                .help("IP to bind")
                .default_value("0.0.0.0"),
        )
        .arg(
            Arg::with_name("port")
                .help("IP to bind")
                .default_value("8080"),
        )
        .get_matches();

    let ip = matches.value_of("ip").unwrap();
    let port = matches.value_of("port").unwrap();

    let conf = Config::new(ip, port).unwrap_or_else(|err| {
        match err {
            ParseError::InvalidIp(err) => eprintln!("The given IP could not be parsed ({})", err),
            ParseError::InvalidPort(err) => {
                eprintln!("The given Port could not be parsed ({})", err)
            }
        };
        process::exit(1);
    });

    redbackup_node::run(conf);
}
