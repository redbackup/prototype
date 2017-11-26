#![recursion_limit="128"] // required for database inference
#[macro_use] extern crate quick_error;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate log;
extern crate futures;
extern crate futures_cpupool;
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
pub mod list;
pub mod restore;
mod chunk_index;

pub use create_backup::config::{CreateBackupConfig, CreateBackupConfigError};
pub use restore::config::{RestoreConfig, RestoreConfigError};
use chrono::prelude::*;

pub fn create_backup(config: config::Config, create_backup_config: CreateBackupConfig) -> Result<(), create_backup::CreateError> {
    create_backup::CreateBackupContext::new(
        config,
        create_backup_config,
    )?.run()
}

pub fn list(config: config::Config) -> Result<Vec<(String, DateTime<Utc>)>, list::ListError> {
    list::List::new(config)?.run()
}

pub fn restore(config: config::Config, restore_config: RestoreConfig) -> Result<(), restore::RestoreError> {
    restore::Restore::new(config, restore_config)?.run()
}
