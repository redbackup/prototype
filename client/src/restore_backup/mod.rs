pub mod config;
pub mod error;
pub mod utils;
pub use self::error::RestoreBackupError;
pub use self::config::RestoreBackupConfig;

use std::io;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use tokio_core;
use tokio_service::Service;
use tokio_proto::TcpClient;
use tokio_core::reactor::{Core, Handle};
use futures::*;

use chrono::prelude::*;

use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::*;

use super::Progress;
use super::config::Config;
use super::chunk_index::ChunkIndex;

/// Implementation of the restore process
pub struct RestoreBackupContext {
    config: Config,
    restore_config: RestoreBackupConfig,
    event_loop: Core,
    handle: Handle,
    progress_sender: Sender<Progress>,
}

impl RestoreBackupContext {
    /// Create initial structures for a restore.
    pub fn new(
        config: Config,
        restore_config: RestoreBackupConfig,
        progress_sender: Sender<Progress>,
    ) -> Result<Self, RestoreBackupError> {
        let event_loop = tokio_core::reactor::Core::new()?;
        let handle = event_loop.handle();

        Ok(Self {
            config,
            restore_config,
            event_loop,
            handle,
            progress_sender,
        })
    }

    /// The restore process
    pub fn run(&mut self) -> Result<(), RestoreBackupError> {

        info!("Restore chunk index");
        let chunk_index = self.restore_chunk_index()?;

        info!("Restore folder structure");
        Self::restore_folder_structure(&self.restore_config.restore_dir, &chunk_index, None)?;

        info!("Restore files");
        self.restore_chunks(&chunk_index)?;

        info!("Successfully finished restoring all files.");
        Ok(())
    }

    /// Reconstruct the chunk index of the specified backup
    fn restore_chunk_index(&mut self) -> Result<ChunkIndex, RestoreBackupError> {
        let chunk_identifier = &self.restore_config.backup_id;
        debug!(
            "Request chunk index {} from node at {}",
            chunk_identifier,
            self.config.addr
        );
        let message = GetChunks::new(vec![chunk_identifier.clone()]);
        let request = TcpClient::new(RedClientProto)
            .connect(&self.config.addr, &self.handle.clone())
            .and_then(|client| client.call(message));
        let response = self.event_loop.run(request).map_err(
            |e| RestoreBackupError::from(e),
        )?;

        let chunks = match response.body {
            MessageKind::ReturnChunks(body) => Some(body.chunks),
            _ => None,
        }.ok_or(RestoreBackupError::NodeCommunicationError)?;

        let chunk: &ChunkContentElement = chunks.get(0).ok_or(
            RestoreBackupError::RootHandleChunkNotAvailable(chunk_identifier.clone()),
        )?;

        let now = Utc::now();
        let path = PathBuf::from(format!("/tmp/{}.db", now.to_rfc3339()));
        utils::restore_file_content(&chunk.chunk_content.as_slice(), &path)?;
        Ok(ChunkIndex::new(path, now)?)
    }

    /// Recreate the whole folder structure of the backup recursively
    fn restore_folder_structure(
        root_folder: &PathBuf,
        chunk_index: &ChunkIndex,
        parent_folder_id: Option<i32>,
    ) -> Result<(), RestoreBackupError> {
        debug!(
            "Request folder by parent id (if any) {:?} from chunk index",
            parent_folder_id
        );
        let folders = chunk_index.get_folders_by_parent(parent_folder_id)?;
        let path = root_folder;

        for folder in folders {
            debug!("Restore folder {:?}", path);
            let mut path = path.clone();
            path.push(&folder.name);
            utils::create_folder(&path)?;
            Self::restore_folder_structure(&path, &chunk_index, Some(folder.id))?;
        }
        Ok(())
    }

    /// Reassemble files from all chunks in the chunk index
    fn restore_chunks(&mut self, chunk_index: &ChunkIndex) -> Result<(), RestoreBackupError> {
        let chunks = chunk_index.get_all_chunks()?;
        let mut progress = Progress::new(self.progress_sender.clone(), chunks.len());
        for chunk in chunks {
            debug!("Request chunk {}", chunk.chunk_identifier);
            let chunk_content = self.request_chunk(chunk.chunk_identifier.clone())?;

            debug!("Restore path for chunk {}", chunk_content.chunk_identifier);
            let mut path = self.restore_config.restore_dir.clone();
            path.push(chunk_index.get_file_path(chunk.file)?);

            utils::restore_file_content(&chunk_content.chunk_content.as_slice(), &path)?;
            debug!("Restored chunk {} to {:?}", chunk.chunk_identifier, path);
            progress.increment();
        }
        Ok(())
    }

    /// Query a chunk from the node by chunk identifier
    fn request_chunk(
        &mut self,
        chunk_identifier: String,
    ) -> Result<ChunkContentElement, RestoreBackupError> {
        let message = GetChunks::new(vec![chunk_identifier.clone()]);
        let request = TcpClient::new(RedClientProto)
            .connect(&self.config.addr, &self.handle.clone())
            .and_then(|client| client.call(message))
            .and_then(|response| match response.body {
                MessageKind::ReturnChunks(body) => Ok(body.chunks),
                _ => Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Chunk not contained in node response",
                )),
            });
        let mut chunk_contents = self.event_loop.run(request).map_err(
            |e| RestoreBackupError::from(e),
        )?;

        chunk_contents.pop().ok_or(
            RestoreBackupError::ChunkNotAvailable(
                chunk_identifier.clone(),
            ),
        )
    }
}
