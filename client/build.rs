extern crate diesel;

use std::env;

use self::diesel::prelude::*;
use self::diesel::migrations;
use self::diesel::sqlite::SqliteConnection;

fn main() {
    prepare_inference_database();
}

/// Prepares a SQLITE database file with migrations to allow schema inference.during build.
///
/// This satisfies the DATABASE_FILE env required by `src/chunk_index/schema.rs`
fn prepare_inference_database() {
    // Requires OUT_DIR env set by the cargo build environment. It contains the build target directory.
    let database_file = format!("{}/database-client.db", env::var("OUT_DIR").unwrap());

    let connection = SqliteConnection::establish(&database_file).expect(&format!(
        "Build Database Error: Connecting to {} failed",
        database_file
    ));

    migrations::run_pending_migrations(&connection).expect(
        "Build Database Error: Migrations unsuccessful",
    );

    println!("cargo:rustc-env=DATABASE_FILE={}", database_file);

}
