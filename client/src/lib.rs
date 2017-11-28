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

#[cfg(test)]
mod tests;

pub mod config;
pub mod create_backup;
mod chunk_index;

pub use create_backup::config::{CreateBackupConfig, CreateBackupConfigError};

pub fn create_backup(config: config::Config, create_backup_config: CreateBackupConfig) -> Result<(), create_backup::CreateError> {
    create_backup::CreateBackupContext::new(
        config,
        create_backup_config,
    )?.run()
}
