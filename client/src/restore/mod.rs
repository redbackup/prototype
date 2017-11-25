pub mod config;
pub mod restore_error;
pub use self::restore_error::RestoreError;
pub use self::config::RestoreConfig;

use std::path::PathBuf;

use tokio_core;
use tokio_service::Service;
use tokio_proto::TcpClient;
use tokio_core::reactor::{Core,Handle};
use futures::*;
use chrono::prelude::*;

use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::*;

use super::config::Config;
use super::chunk_index::{ChunkIndex, DatabaseError};
use super::chunk_index::schema::{Folder, NewFolder, File, NewFile, NewChunk, Chunk};

pub struct Restore {
    config: Config,
    restore_config: RestoreConfig,
    chunk_index: Option<ChunkIndex>,
    event_loop: Core,
    handle: Handle,
}

impl Restore {
    /// Create initial structures for a restore.
    pub fn new(config: Config, restore_config: RestoreConfig) -> Result<Self,RestoreError> {
        let event_loop = tokio_core::reactor::Core::new()?;
        let handle = event_loop.handle();

        Ok(Self {
            config,
            restore_config,
            chunk_index: None,
            event_loop,
            handle,
        })
    }

    /// The restore process
    pub fn run(&mut self) -> Result<(), RestoreError> {
        unimplemented!()
    }
}
