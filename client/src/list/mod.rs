pub mod list_error;
pub use self::list_error::ListError;

use tokio_core;
use tokio_service::Service;
use tokio_proto::TcpClient;
use tokio_core::reactor::{Core,Handle};
use futures::*;

use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::*;

use super::config::Config;

pub struct List {
    config: Config,
    event_loop: Core,
    handle: Handle,
}

impl List {
    /// Create initial structures to list backups
    pub fn new(config: Config) -> Result<Self,ListError> {
        let event_loop = tokio_core::reactor::Core::new()?;
        let handle = event_loop.handle();

        Ok(Self {
            config,
            event_loop,
            handle,
        })
    }

    /// The list process
    pub fn run(&mut self) -> Result<(), ListError> {
        unimplemented!()
    }
}
