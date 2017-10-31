extern crate redbackup_management;
extern crate clap;
#[macro_use]
extern crate version;

use clap::App;

fn main() {
    App::new("redbackup management-cli")
       .version(version!())
       .about("redbackup management server")
       .author("Fabian Hauser & Raphael Zimmermann")
       .get_matches();
    redbackup_management::hello_world()
}
