pub mod list_error;
pub use self::list_error::ListError;

use tokio_core;
use tokio_service::Service;
use tokio_proto::TcpClient;
use tokio_core::reactor::{Core,Handle};
use futures::*;
use chrono::prelude::*;

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

    pub fn run(&mut self) -> Result<Vec<(String, DateTime<Utc>)>, ListError> {
        Ok(self.get_root_handles()?.iter().map(|e| (e.chunk_identifier.clone(), e.expiration_date)).collect())
    }

    fn get_root_handles(&mut self) -> Result<Vec<ChunkContentElement>,ListError> {
        self.message_node_sync(GetRootHandles::new())
            .map(|res| match res.body {
                MessageKind::ReturnRootHandles(body) => Ok(body.root_handle_chunks),
                _ => Err(ListError::NodeCommunicationError),
            })?
    }

    /// Send a `Message` to the node.
    fn message_node_sync(&mut self, message: Message) -> Result<Message, ListError> {
        let future = TcpClient::new(RedClientProto)
            .connect(&self.config.addr, &self.handle.clone())
            .and_then(|client| client.call(message));
        self.event_loop.run(future).map_err(|e| ListError::from(e))
    }

}
