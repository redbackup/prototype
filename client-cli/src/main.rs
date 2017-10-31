extern crate redbackup_client;
extern crate clap;
#[macro_use]
extern crate version;

use clap::App;

fn main() {
    App::new("redbackup client-cli")
       .version(version!())
       .about("redbackup client")
       .author("Fabian Hauser & Raphael Zimmermann")
       .get_matches();
    redbackup_client::hello_world()
}
