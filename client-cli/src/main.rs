#[macro_use]
extern crate clap;
extern crate redbackup_client;

use std::process;

use redbackup_client::config::{Config, ParseError};

use clap::App;


fn main() {
    App::new("redbackup client-cli")
        .about("redbackup client")
        .version(crate_version!())
        .author(crate_authors!())
        .get_matches();

    let config = Config::new("0.0.0.0", "8080").unwrap_or_else(|err| {
        match err {
            ParseError::InvalidIp(err) => eprintln!("The given IP could not be parsed ({})", err),
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
