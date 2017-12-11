#![recursion_limit = "128"] // required for database inference
extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate dns_lookup;
extern crate futures;
extern crate futures_cpupool;
#[macro_use]
extern crate log;
#[macro_use]
extern crate quick_error;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate sha2;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate uuid;
extern crate glob;

extern crate redbackup_protocol;

#[cfg(test)]
mod tests;
pub mod config;
pub mod progress;
pub mod create_backup;
pub mod list_backups;
pub mod restore_backup;
mod chunk_index;

use std::sync::mpsc::Sender;

pub use create_backup::config::{CreateBackupConfig, CreateBackupConfigError};
pub use restore_backup::config::{RestoreBackupConfig, RestoreBackupConfigError};
pub use progress::Progress;
use chrono::prelude::*;

pub fn create_backup(
    config: config::Config,
    create_backup_config: CreateBackupConfig,
    progress_sender: Sender<Progress>,
) -> Result<(), create_backup::CreateError> {
    create_backup::CreateBackupContext::new(config, create_backup_config, progress_sender)?
        .run()
}

pub fn list_backups(
    config: config::Config,
) -> Result<Vec<(String, DateTime<Utc>)>, list_backups::ListBackupsError> {
    list_backups::ListBackupsContext::new(config)?.run()
}

pub fn restore_backup(
    config: config::Config,
    restore_backup_config: RestoreBackupConfig,
    progress_sender: Sender<Progress>,
) -> Result<(), restore_backup::RestoreBackupError> {
    restore_backup::RestoreBackupContext::new(config, restore_backup_config, progress_sender)?
        .run()
}
