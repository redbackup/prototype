pub mod create_chunk_index;
pub mod config;
pub mod create_error;
pub mod create_utils;
pub use self::create_error::CreateError;
pub use self::config::CreateBackupConfig;

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
use self::create_chunk_index::CreateChunkIndex;

pub struct CreateBackupContext {
    config: Config,
    create_backup_config: CreateBackupConfig,
    chunk_index: ChunkIndex,
    event_loop: Core,
    handle: Handle,
}

impl CreateBackupContext {
    /// Create initial structures for a new backup.
    pub fn new(config: Config, create_backup_config: CreateBackupConfig) -> Result<Self,CreateError> {
        let now = Utc::now();
        let chunk_index_file = PathBuf::from(format!("{}/chunk_index-{}.db",
            config.chunk_index_storage.to_str().unwrap(),
            now.to_rfc3339()));

        let event_loop = tokio_core::reactor::Core::new()?;
        let handle = event_loop.handle();

        debug!("Create chunk index {}", chunk_index_file.to_string_lossy());
        Ok(Self {
            config,
            create_backup_config,
            chunk_index: ChunkIndex::new(chunk_index_file, now)?,
            event_loop,
            handle,
        })
    }

    /// The backup process
    pub fn run(&mut self) -> Result<(), CreateError> {
        info!("Create chunk index from {:?}", self.create_backup_config.backup_dir);
        CreateChunkIndex::new(&self.chunk_index, &self.create_backup_config.backup_dir)?;

        info!("Request designation from node");
        self.request_designation()?;
        info!("Designation was granted by the node");

        info!("Check which chunks are already on the node");
        debug!("Collecting chunks from database");
        let mut chunks = self.chunk_index.get_all_chunks()?;
        let chunk_elements = chunks.iter().map(|e| self.chunk_to_chunk_element(e)).collect();

        debug!("Get available chunks from node");
        let node_chunks = self.get_available_chunks_from_node(chunk_elements)?;
        info!(
            "{} of total {} chunks are not yet stored on the node",
            chunks.len() - node_chunks.len(),
            chunks.len()
        );
        Self::reduce_by_remaining_chunks(&mut chunks, &node_chunks);

        info!("Send chunks to node");
        for chunk in chunks {
            debug!("Collect chunk {} content", chunk.chunk_identifier);
            let chunk_ce = self.chunk_to_chunk_content_element(&chunk)?;
            debug!("Send chunk {} to node", chunk_ce.chunk_identifier);
            self.send_chunk(chunk_ce)?;
            debug!("Successfully sent chunk {} to node", chunk.chunk_identifier);
        }
        info!("Successfully sent all data chunks.");

        info!("Send chunk index to node as root handle");
        self.send_chunk_index()?;
        info!("Successfully sent chunk index");

        Ok(())
    }

    /// Send a backup designation to the node
    fn request_designation(&mut self) -> Result<(),CreateError> {
        let expiration_date = self.create_backup_config.expiration_date.clone();
        let req = GetDesignation::new(0, expiration_date);
        let designation = self.message_node_sync(req)
            .map(|res| match res.body {
                MessageKind::ReturnDesignation(body) => Ok(body.designation),
                _ => Err(CreateError::NodeCommunicationError),
            })?;

        match designation {
            Ok(true) => Ok(()),
            Ok(false) => Err(CreateError::DesignationNotGrantedError(format!("{:?}", self.config.addr))),
            Err(e) => Err(e),
        }
    }

    /// Ask the node, which chunks of `chunk_elements` he already has.
    fn get_available_chunks_from_node(
        &mut self,
        chunk_elements: Vec<ChunkElement>
    ) -> Result<Vec<ChunkElement>,CreateError> {
        let req = GetChunkStates::new(chunk_elements);
        self.message_node_sync(req)
            .map(|res| match res.body {
                MessageKind::ReturnChunkStates(body) => Ok(body.chunks),
                _ => Err(CreateError::NodeCommunicationError),
            })?
    }

    /// Remove items in vector `reduction` from vector `elements` by the chun_identifier.
    fn reduce_by_remaining_chunks(elements: &mut Vec<Chunk>, reduction: &Vec<ChunkElement>) {
        elements.retain(|e| {
                reduction.iter().filter(|x| e.chunk_identifier == x.chunk_identifier)
                .count() == 0
            });
    }

    /// Send the `chunk` to the node, raising an Error if the chunk was not acknowledged.
    fn send_chunk(&mut self, chunk: ChunkContentElement) -> Result<(), CreateError> {
        let chunk_identifier = chunk.chunk_identifier.clone();

        let req = PostChunks::new(vec!(chunk));
        let acknowledged_chunks: Vec<ChunkElement> = self.message_node_sync(req)
            .map(|res| match res.body {
                MessageKind::AcknowledgeChunks(body) => Some(body.chunks),
                _ => None,
            })?
            .ok_or(CreateError::NodeCommunicationError)?;

        let acknowledged_chunk: &ChunkElement = acknowledged_chunks.get(0)
            .ok_or(CreateError::ChunkNotAcknowledged(chunk_identifier.clone()))?;

        if acknowledged_chunk.chunk_identifier == chunk_identifier {
            debug!("Acknowledged chunk: {}", acknowledged_chunk.chunk_identifier);
            Ok(())
        } else {
            error!(
                "Expected chunk {} acknowledgement from node, got {}",
                chunk_identifier,
                acknowledged_chunk.chunk_identifier
            );
            Err(CreateError::ChunkNotAcknowledged(chunk_identifier))
        }
    }

    /// Send the chunk index as root_handle to the node.
    fn send_chunk_index(&mut self) -> Result<(), CreateError> {
        debug!("Collect metadata and file content of chunk index");
        let file_name = self.chunk_index.get_file_name();
        let chunk_identifier = create_utils::file_hash(&file_name)?;
        let chunk_content = create_utils::read_file_content(&file_name)?;
        let expiration_date = self.create_backup_config.expiration_date.clone();
        debug!("Send chunk index {} to node", chunk_identifier);
        self.send_chunk(ChunkContentElement{
            chunk_identifier,
            chunk_content,
            expiration_date,
            root_handle: true,
        })
    }

    /// Send a `Message` to the node.
    fn message_node_sync(&mut self, message: Message) -> Result<Message, CreateError> {
        let future = TcpClient::new(RedClientProto)
            .connect(&self.config.addr, &self.handle.clone())
            .and_then(|client| client.call(message));
        self.event_loop.run(future).map_err(|e| CreateError::from(e))
    }


    /// Map a `Chunk` to a `ChunkContentElement` enriched with the file content.
    fn chunk_to_chunk_content_element(
        &self, chunk: &Chunk
    ) -> Result<ChunkContentElement, CreateError> {

        // Get full path of this chunk's file
        let mut path = self.create_backup_config.backup_dir.clone();
        path.pop(); // The last folder here is the same as the root folder of the file
        path.push(self.chunk_index.get_file_path(chunk.file)?);

        Ok(ChunkContentElement {
            chunk_identifier: chunk.chunk_identifier.clone(),
            expiration_date: self.create_backup_config.expiration_date.clone(),
            root_handle: false,
            chunk_content: create_utils::read_file_content(&path)?,
        })
    }

    /// Map a `Chunk` to a `ChunkElement`
    fn chunk_to_chunk_element(&self, chunk: &Chunk) -> ChunkElement {
        ChunkElement {
            chunk_identifier: chunk.chunk_identifier.clone(),
            expiration_date: self.create_backup_config.expiration_date.clone(),
            root_handle: false,
        }
    }
}
