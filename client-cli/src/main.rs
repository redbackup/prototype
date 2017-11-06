#[macro_use]
extern crate clap;
extern crate redbackup_client;

use std::process;

use redbackup_client::config::{Config, ParseError};

use clap::{App, Arg};


fn main() {
    let matches = App::new("redbackup client-cli")
        .about("redbackup client")
        .version(crate_version!())
        .author(crate_authors!())
         .arg(
            Arg::with_name("node-hostname")
                .help("hostname of the node to contact")
                .default_value("localhost"),
        )
        .arg(
            Arg::with_name("node-port")
                .help("port of the node to contact")
                .default_value("8080"),
        )
        .get_matches();

    let node_host = matches.value_of("node-hostname").unwrap();
    let node_port = matches.value_of("node-port").unwrap();

    let config = Config::new(node_host, node_port).unwrap_or_else(|err| {
        match err {
            ParseError::InvalidHostname(err) => eprintln!("The given hostname is invalid ({})", err),
            ParseError::InvalidPort(err) => {
                eprintln!("The given Port could not be parsed ({})", err)
            }
        };
        process::exit(1);
    });


    redbackup_client::backup(config).unwrap_or_else(|err| {
        eprintln!("Huston, we have a problem! ({})", err)
    });
}
