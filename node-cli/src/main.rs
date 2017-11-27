#[macro_use]
extern crate clap;
extern crate env_logger;
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
            Arg::with_name("known-node")
                .short("k")
                .long("known-node")
                .multiple(true)
                .number_of_values(1)
                .takes_value(true)
                .help(
                    "ip address and port (<ip-address> <port>) of other known nodes in the network",
                ),
        )
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
        .arg(
            Arg::with_name("storage-dir")
                .help("path to the storage directory")
                .default_value("./data/"),
        )
        .arg(
            Arg::with_name("db-file")
                .help("path to the database file")
                .default_value("db.sqlite3"),
        )
        .get_matches();

    let ip = matches.value_of("ip").unwrap();
    let port = matches.value_of("port").unwrap();
    let storage_dir = matches.value_of("storage-dir").unwrap();
    let db_file = matches.value_of("db-file").unwrap();
    let known_nodes = matches
        .values_of("known-node")
        .unwrap_or_default()
        .map(|v| v.to_owned())
        .collect::<Vec<_>>();

    let conf = Config::new(ip, port, storage_dir, db_file, known_nodes).unwrap_or_else(|err| {
        match err {
            ParseError::InvalidIp(err) => eprintln!("The given IP could not be parsed ({})", err),
            ParseError::InvalidPort(err) => {
                eprintln!("The given Port could not be parsed ({})", err)
            }
        };
        process::exit(1);
    });

    env_logger::init().unwrap();
    redbackup_node::run(conf);
}
