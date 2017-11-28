pub mod error;
pub use self::error::ListBackupsError;

use tokio_core;
use tokio_service::Service;
use tokio_proto::TcpClient;
use tokio_core::reactor::{Core,Handle};
use futures::*;
use chrono::prelude::*;

use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::*;

use super::config::Config;

pub struct ListBackupsContext {
    config: Config,
    event_loop: Core,
    handle: Handle,
}

impl ListBackupsContext {
    pub fn new(config: Config) -> Result<Self,ListBackupsError> {
        let event_loop = tokio_core::reactor::Core::new()?;
        let handle = event_loop.handle();

        Ok(Self {
            config,
            event_loop,
            handle,
        })
    }

    pub fn run(&mut self) -> Result<Vec<(String, DateTime<Utc>)>, ListBackupsError> {
        Ok(self.get_root_handles()?.iter().map(|e| (e.chunk_identifier.clone(), e.expiration_date)).collect())
    }

    fn get_root_handles(&mut self) -> Result<Vec<ChunkContentElement>,ListBackupsError> {
        self.message_node_sync(GetRootHandles::new())
            .map(|res| match res.body {
                MessageKind::ReturnRootHandles(body) => Ok(body.root_handle_chunks),
                _ => Err(ListBackupsError::NodeCommunicationError),
            })?
    }

    /// Send a `Message` to the node.
    fn message_node_sync(&mut self, message: Message) -> Result<Message, ListBackupsError> {
        let future = TcpClient::new(RedClientProto)
            .connect(&self.config.addr, &self.handle.clone())
            .and_then(|client| client.call(message));
        self.event_loop.run(future).map_err(|e| ListBackupsError::from(e))
    }
}
