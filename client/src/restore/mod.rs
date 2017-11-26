pub mod config;
pub mod restore_error;
mod restore_utils;
pub use self::restore_error::RestoreError;
pub use self::config::RestoreConfig;

use std::io;
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
use super::chunk_index::ChunkIndex;

pub struct Restore {
    config: Config,
    restore_config: RestoreConfig,
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
            event_loop,
            handle,
        })
    }

    /// The restore process
    pub fn run(&mut self) -> Result<(), RestoreError> {

        info!("Restoring chunk index");
        let chunk_index = self.restore_chunk_index()?;

        info!("Restoring folder structure");
        Self::restore_folder_structure(&self.restore_config.restore_dir, &chunk_index, None)?;

        info!("Restoring");
        self.request_chunks(&chunk_index)?;
//        self.restore_files(&chunk_index, chunks)?;

        Ok(())
    }


    fn restore_chunk_index(&mut self) -> Result<ChunkIndex,RestoreError> {
        let message = GetChunks::new(vec!(self.restore_config.backup_id.clone()));
        let request = TcpClient::new(RedClientProto)
            .connect(&self.config.addr, &self.handle.clone())
            .and_then(|client| client.call(message));
        let response = self.event_loop.run(request).map_err(|e| RestoreError::from(e))?;
        let chunks = match response.body {
                MessageKind::ReturnChunks(body) => Some(body.chunks),
                    _ => None,
                }.ok_or(RestoreError::NodeCommunicationError)?;

        let chunk: &ChunkContentElement = chunks.get(0)
            .ok_or(RestoreError::RootHandleChunkNotAvailable(self.restore_config.backup_id.clone()))?;

        let now = Utc::now();
        let path = PathBuf::from(format!("/tmp/{}.db", now.to_rfc3339()));
        restore_utils::restore_file_content(&chunk.chunk_content.as_slice(), &path)?;
        Ok(ChunkIndex::new(path, now)?)
    }

    fn restore_folder_structure(root_folder: &PathBuf, chunk_index: &ChunkIndex, parent_folder_id: Option<i32>) -> Result<(),RestoreError> {
        let folders = chunk_index.get_folders_by_parent(parent_folder_id)?;
        let path = root_folder;

        for folder in folders {
            let mut path = path.clone();
            path.push(&folder.name);
            restore_utils::create_folder(&path)?;
            debug!("Restored folder {:?}", path);
            Self::restore_folder_structure(&path, &chunk_index, Some(folder.id))?;
        }
        Ok(())
    }

    fn request_chunks(
        &mut self,
        chunk_index: &ChunkIndex
    ) -> Result<(), RestoreError>{

        let chunks = chunk_index.get_all_chunks()?;

        for chunk in chunks {
            let message = GetChunks::new(vec!(chunk.chunk_identifier.clone()));
            let request = TcpClient::new(RedClientProto)
                .connect(&self.config.addr, &self.handle.clone())
                .and_then(|client| client.call(message))
                .and_then(|response|{
                    match response.body {
                        MessageKind::ReturnChunks(body) => Ok(body.chunks),
                            _ => Err(io::Error::new(io::ErrorKind::Other, "Chunk not contained in node response")),
                        }
                });
            let chunk_contents = self.event_loop.run(request).map_err(|e| RestoreError::from(e))?;
            // TODO: Refactor the following
            let chunk_content = chunk_contents.get(0).ok_or(RestoreError::ChunkNotAvailable(chunk.chunk_identifier.clone()))?;
            let mut path = self.restore_config.restore_dir.clone();
            chunk_index.get_full_chunk_path(chunk.file)?.iter().for_each(|e| path.push(e));
            restore_utils::restore_file_content(&chunk_content.chunk_content.as_slice(), &path)?;
        }

        Ok(())
    }
/*
    fn restore_files(
        &mut self,
        chunk_index: &ChunkIndex,
        chunks: Vec<Box<Future<Item=ChunkContentElement, Error=RestoreError>>>
    ) -> Result<(), RestoreError>{
        unimplemented!()
    }
    */
}
