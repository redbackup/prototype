pub mod chunk_index_builder;
pub mod config;

use std::io;
use std::io::Read;
use std::fs;

use tokio_core;
use tokio_service::Service;
use tokio_proto::TcpClient;
use futures::*;
use chrono::prelude::*;

use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::{MessageKind, GetDesignation, GetChunkStates, ChunkElement, PostChunks, ChunkContentElement};
use super::config::Config;
use super::chunk_index::{ChunkIndex, DatabaseError};
use super::chunk_index::schema::{Folder, NewFolder, File, NewFile, NewChunk};
use self::chunk_index_builder::{ChunkIndexBuilder, BuilderError};

quick_error!{
    #[derive(Debug)]
    pub enum CreateError {
        DatabaseError(err: DatabaseError) {
            from()
            cause(err)
        }
        BuilderError(err: BuilderError) {
            from()
            cause(err)
        }
        IoError(err: io::Error) {
            from()
            cause(err)
        }
        DesignationNotGrantedError(node: String) {
            description("Designation was not granted")
            display("Designation was not granted by the node {}", node)
        }
    }
}

pub fn run(config: Config, create_config: config::CreateConfig) -> Result<(), CreateError> {
    let now = Utc::now();
    let file_name = format!("{}/chunk_index-{}.db",
        config.chunk_index_storage.to_str().unwrap(),
        now.to_rfc3339());
    let chunk_index = ChunkIndex::new(&file_name, now)?;
    info!("Created chunk index {}", file_name);

    let builder = ChunkIndexBuilder::new(&chunk_index, &create_config.backup_dir)?;
    builder.build()?;

    let mut event_loop = tokio_core::reactor::Core::new()?;
    let handle = event_loop.handle();

    let chunks = chunk_index.get_all_chunks()?;
    let expiration_date_clone = create_config.expiration_date.clone();

    let chunk_elements = chunks.iter()
        .map(move|e|{
            ChunkElement {
                chunk_identifier: e.chunk_identifier.clone(),
                expiration_date: expiration_date_clone,
                root_handle: false,
            }
        }).collect();

    let expiration_date_clone = create_config.expiration_date.clone();
    let designation_request = TcpClient::new(RedClientProto)
        .connect(&config.addr, &handle.clone())
        .and_then(move|client| {
            info!("Sending GetDesignation message");
            client.call(GetDesignation::new(0, expiration_date_clone))
        })
        .map(|res| match res.body {
            MessageKind::ReturnDesignation(body) => Some(body.designation),
            _ => None,
        }).map(|c| c.unwrap());
    let designation = event_loop.run(designation_request).map_err(|e| CreateError::from(e))?;

    if !designation {
        CreateError::DesignationNotGrantedError(format!("{:?}", config.addr));
    } else {
        info!("Designation was granted by the node");
    }

    let node_chunks_future = TcpClient::new(RedClientProto)
        .connect(&config.addr, &handle.clone())
        .and_then(move|client| {
            info!("Sending GetChunkStates message");
            client.call(GetChunkStates::new(chunk_elements))
        })
        .map(|res| match res.body {
            MessageKind::ReturnChunkStates(body) => Some(body.chunks),
            _ => None,
        }).map(|c| c.unwrap());
    let node_chunks: Vec<ChunkElement> = event_loop.run(node_chunks_future).map_err(|e| CreateError::from(e))?;

    let mut remaining_chunks = chunks.clone();
    remaining_chunks.retain(|x| {
            node_chunks.iter().filter(|e| e.chunk_identifier == x.chunk_identifier)
            .count() == 0
        });
    let remaining_chunks = remaining_chunks;
    info!("{} of total {} chunks are already stored on the node", chunks.len() - remaining_chunks.len(), chunks.len());

    for chunk in remaining_chunks.iter() {
        let mut path = create_config.backup_dir.clone();
        path.pop();
        chunk_index.get_full_chunk_path(chunk.file)?.iter().for_each(|x| path.push(x));
        let path = path;
        info!("Sending file {:?}", path);

        let mut fhandle = fs::File::open(path)?;
        let mut buf = Vec::new();
        fhandle.read_to_end(&mut buf)?;

        let chunk_content_element = ChunkContentElement{
            chunk_identifier: chunk.chunk_identifier.clone(),
            expiration_date: create_config.expiration_date.clone(),
            root_handle: false,
            chunk_content: buf,
        };

        info!("Sending chunk message for {}", chunk.chunk_identifier);
        let node_post_chunks = TcpClient::new(RedClientProto)
            .connect(&config.addr, &handle.clone())
            .and_then(move|client| {
                info!("Sending PostChunks message");
                client.call(PostChunks::new(vec!(chunk_content_element)))
            })
            .map(|res| match res.body {
                MessageKind::AcknowledgeChunks(body) => Some(body.chunks),
                _ => None,
            });

        let acked_chunks = event_loop.run(node_post_chunks).map_err(|e| CreateError::from(e))?;

        if let Some(acked_chunks) = acked_chunks {
            let acked_identifier = acked_chunks.get(0).unwrap().chunk_identifier.clone();
            if acked_identifier == chunk.chunk_identifier {
                debug!("Acked chunk: {}", acked_identifier);
            } else {
                error!("Expected chunk {} acknowledgement, got {}", chunk.chunk_identifier, acked_identifier);
            }
        } else {
            error!("Chunk {} was not acknowledged by the node!", chunk.chunk_identifier);
        }
    }

    Ok(())
}
