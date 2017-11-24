pub mod chunk_index_builder;
pub mod config;
pub mod create_error;
pub mod create_utils;
pub use self::create_error::CreateError;
pub use self::config::CreateConfig;

use std::path::PathBuf;

use tokio_core;
use tokio_service::Service;
use tokio_proto::TcpClient;
use tokio_core::reactor::{Core,Handle};
use futures::*;
use chrono::prelude::*;

use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::{MessageKind, GetDesignation, GetChunkStates, ChunkElement, PostChunks, ChunkContentElement};

use super::config::Config;
use super::chunk_index::{ChunkIndex, DatabaseError};
use super::chunk_index::schema::{Folder, NewFolder, File, NewFile, NewChunk, Chunk};
use self::chunk_index_builder::ChunkIndexBuilder;

pub struct Create {
    config: Config,
    create_config: CreateConfig,
    chunk_index_file: PathBuf,
    chunk_index: ChunkIndex,
    event_loop: Core,
    handle: Handle,
}

impl Create {
    /// Create initial structures for a new backup.
    pub fn new(config: Config, create_config: CreateConfig) -> Result<Self,CreateError> {
        let now = Utc::now();
        let chunk_index_file = PathBuf::from(format!("{}/chunk_index-{}.db",
            config.chunk_index_storage.to_str().unwrap(),
            now.to_rfc3339()));
        info!("Creating chunk index {}", chunk_index_file.to_string_lossy());

        let event_loop = tokio_core::reactor::Core::new()?;
        let handle = event_loop.handle();

        Ok(Self {
            config,
            create_config,
            chunk_index_file: chunk_index_file.clone(),
            chunk_index: ChunkIndex::new(chunk_index_file, now)?,
            event_loop,
            handle,
        })
    }

    /// The backup process
    pub fn run(&mut self) -> Result<(), CreateError> {
        // Read folder structure
        let builder = ChunkIndexBuilder::new(&self.chunk_index, &self.create_config.backup_dir)?;
        builder.build()?;

        self.request_designation()?;
        info!("Designation was granted by the node");

        let mut chunks = self.chunk_index.get_all_chunks()?;
        let chunk_elements = chunks.iter()
            .map(|e|{
                ChunkElement {
                    chunk_identifier: e.chunk_identifier.clone(),
                    expiration_date: self.create_config.expiration_date.clone(),
                    root_handle: false,
                }
            }).collect();


        let node_chunks = self.get_available_chunks_from_node(chunk_elements)?;
        info!(
            "{} of total {} chunks are already stored on the node",
            chunks.len() - node_chunks.len(),
            chunks.len()
        );
        Self::reduce_by_remaining_chunks(&mut chunks, &node_chunks);


        // Send chunks one by one.
        for chunk in chunks {
            let chunk_content_element = self.chunk_to_chunk_content_element(chunk)?;
            info!("Sending chunk message for {}", chunk_content_element.chunk_identifier);
            self.send_chunk(chunk_content_element)?;
        }
        info!("Successfully sent all data chunks.");

        self.send_chunk_index()?;
        info!("Successfully sent chunk index as root handle.");

        Ok(())
    }

    /// Send a backup designation to the node
    fn request_designation(&mut self) -> Result<(),CreateError> {
        let expiration_date_clone = self.create_config.expiration_date.clone();
        let addr_clone = self.config.addr.clone();
        let request = TcpClient::new(RedClientProto)
            .connect(&self.config.addr, &self.handle.clone())
            .and_then(move|client| {
                info!("Request designation from node");
                client.call(GetDesignation::new(0, expiration_date_clone))
            })
            .map(|res| match res.body {
                MessageKind::ReturnDesignation(body) => Ok(body.designation),
                _ => Err(CreateError::NodeCommunicationError),
            });

        let designation = self.event_loop.run(request).map_err(|e| CreateError::from(e))??;
        if designation {
            Ok(())
        } else {
            Err(CreateError::DesignationNotGrantedError(format!("{:?}", addr_clone)))
        }
    }

    /// Ask the node, which chunks of `chunk_elements` he already has.
    fn get_available_chunks_from_node(
        &mut self,
        chunk_elements: Vec<ChunkElement>
    ) -> Result<Vec<ChunkElement>,CreateError> {
        let request = TcpClient::new(RedClientProto)
            .connect(&self.config.addr, &self.handle.clone())
            .and_then(move|client| {
                info!("Sending GetChunkStates message");
                client.call(GetChunkStates::new(chunk_elements))
            })
            .map(|res| match res.body {
                MessageKind::ReturnChunkStates(body) => Ok(body.chunks),
                _ => Err(CreateError::NodeCommunicationError),
            });

        let node_chunks = self.event_loop.run(request).map_err(|e| CreateError::from(e))??;
        Ok(node_chunks)
    }

    /// Remove `reduction` from `elements` chunks.
    fn reduce_by_remaining_chunks(elements: &mut Vec<Chunk>, reduction: &Vec<ChunkElement>) {
        elements.retain(|e| {
                reduction.iter().filter(|x| e.chunk_identifier == x.chunk_identifier)
                .count() == 0
            });
    }

    /// Map a `Chunk` to a `ChunkContentElement` enriched with the file content.
    fn chunk_to_chunk_content_element(
        &self, chunk: Chunk
    ) -> Result<ChunkContentElement, CreateError> {

        // Get full path of this chunk's file
        let mut path = self.create_config.backup_dir.clone();
        self.chunk_index.get_full_chunk_path(chunk.file)?.iter().skip(1).for_each(|x| path.push(x));

        Ok(ChunkContentElement {
            chunk_identifier: chunk.chunk_identifier.clone(),
            expiration_date: self.create_config.expiration_date.clone(),
            root_handle: false,
            chunk_content: create_utils::read_file_content(&path)?,
        })

    }

    /// Send the `chunk` to the node, raising an Error if the chunk was not acknowledged.
    fn send_chunk(
        &mut self, chunk: ChunkContentElement
    ) -> Result<(), CreateError> {
        let chunk_identifier = chunk.chunk_identifier.clone();
        let node_post_chunks = TcpClient::new(RedClientProto)
            .connect(&self.config.addr, &self.handle.clone())
            .and_then(|client| {
                info!("Sending PostChunks message");
                client.call(PostChunks::new(vec!(chunk)))
            })
            .map(|res| match res.body {
                MessageKind::AcknowledgeChunks(body) => Some(body.chunks),
                _ => None,
            });

        let acknowledged_chunks: Vec<ChunkElement> = self.event_loop.run(node_post_chunks)
            .map_err(|e| CreateError::from(e))?
            .ok_or(CreateError::NodeCommunicationError)?;

        let acknowledged_chunk: &ChunkElement = acknowledged_chunks.get(0)
            .ok_or(CreateError::ChunkNotAcknowledged(chunk_identifier.clone()))?;

        if acknowledged_chunk.chunk_identifier == chunk_identifier {
            debug!("Acked chunk: {}", acknowledged_chunk.chunk_identifier);
            Ok(())
        } else {
            error!(
                "Expected chunk {} acknowledgement, got {}",
                chunk_identifier,
                acknowledged_chunk.chunk_identifier
            );
            Err(CreateError::ChunkNotAcknowledged(chunk_identifier))
        }
    }

    /// Send the chunk index as root_handle to the node.
    fn send_chunk_index(&mut self) -> Result<(), CreateError> {
        let chunk_identifier = create_utils::file_hash(&self.chunk_index_file)?;
        let chunk_content = create_utils::read_file_content(&self.chunk_index_file)?;
        let expiration_date = self.create_config.expiration_date.clone();
        self.send_chunk(ChunkContentElement{
            chunk_identifier,
            chunk_content,
            expiration_date,
            root_handle: true,
        })
    }
}