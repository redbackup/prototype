extern crate redbackup_node;
extern crate clap;
#[macro_use]
extern crate version;

use clap::App;

fn main() {
    App::new("redbackup node-cli")
       .version(version!())
       .about("redbackup node server")
       .author("Fabian Hauser & Raphael Zimmermann")
       .get_matches();
    redbackup_node::hello_world();
}
