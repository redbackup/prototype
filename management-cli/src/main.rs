extern crate redbackup_management;
#[macro_use]
extern crate clap;

use clap::App;

fn main() {
    App::new("redbackup management-cli")
       .about("redbackup management server")
        .version(crate_version!())
        .author(crate_authors!())
       .get_matches();
    redbackup_management::hello_world()
}
