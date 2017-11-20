#![recursion_limit="128"] // required for database inference
#[macro_use] extern crate quick_error;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate log;
extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_core;
extern crate chrono;
extern crate uuid;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate dns_lookup;
extern crate sha2;

extern crate redbackup_protocol;

pub mod config;
mod chunk_index;
mod create;

use std::path::PathBuf;

pub fn create(config: config::Config, backup_dir: PathBuf) -> Result<(), create::CreateError> {
    create::run(config, backup_dir)
}
