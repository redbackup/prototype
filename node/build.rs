extern crate redbackup_database_helpers;

use std::env;

use redbackup_database_helpers::build_sqlite;

fn main() {
    let database_file = format!("{}/database-node.db", env::var("OUT_DIR").unwrap());
    build_sqlite::build_database_env(&database_file);
}
