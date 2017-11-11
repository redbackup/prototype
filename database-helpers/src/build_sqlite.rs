extern crate diesel;

use self::diesel::prelude::*;
use self::diesel::migrations;
use self::diesel::sqlite::SqliteConnection;

/// Prepares the `DATAGASE_FILE` build environment
pub fn build_database_env(database_file: &String) {
    prepare_database(&database_file);
    println!("cargo:rustc-env=DATABASE_FILE={}", database_file);
}

/// Initialises the `database_file` and runs all migrations on it
fn prepare_database(database_file: &String) {
    let connection = SqliteConnection::establish(&database_file).expect(
        &format!(
            "Build Database Error: Connecting to {} failed",
            database_file
        ),
    );
    migrations::run_pending_migrations(&connection).expect("Build Database Error: Migrations unsuccessful");

}
