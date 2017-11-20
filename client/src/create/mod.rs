mod chunk_index_builder;

use std::io;
use std::path::PathBuf;

use tokio_core;
use tokio_service::Service;
use tokio_proto::TcpClient;
use futures::*;
use chrono::prelude::*;

use redbackup_protocol::RedClientProto;
use redbackup_protocol::message::{MessageKind, GetDesignation, GetChunkStates, ChunkElement};
use super::config::Config;
use super::chunk_index::{ChunkIndex, DatabaseError};
use super::chunk_index::schema::{Folder, NewFolder, File, NewFile, NewChunk, Chunk};
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
    }
}

pub fn run(config: Config, backup_dir: PathBuf) -> Result<(), CreateError> {
    let expiration_date = Utc::now(); // TODO: Command line parameter!
    let now = Utc::now();
    let file_name = format!("{}/chunk_index-{}.db",
        config.chunk_index_storage.to_str().unwrap(),
        now.to_rfc3339());
    let chunk_index = ChunkIndex::new(&file_name, now)?;
    info!("Created chunk index {}", file_name);

    let builder = ChunkIndexBuilder::new(&chunk_index, &backup_dir)?;
    builder.build()?;

    let mut event_loop = tokio_core::reactor::Core::new()?;
    let handle = event_loop.handle();

    let chunks = chunk_index.get_all_chunks()?;
    let expiration_date_clone = expiration_date.clone();

    let available_node_chunks = stream::iter_ok::<_, ()>(chunks)
        .map(move|e|{
            ChunkElement {
                chunk_identifier: e.chunk_identifier.clone(),
                expiration_date: expiration_date_clone,
                root_handle: false,
            }
        })
        .chunks(100)
        .then(|chunks| {
           TcpClient::new(RedClientProto)
            .connect(&config.addr, &handle.clone())
            .and_then(move|client| {
                info!("Sending GetChunkStates message");
                client.call(GetChunkStates::new(chunks.unwrap()))
            })
        }).map(|res| match res.body {
            MessageKind::ReturnChunkStates(body) => Some(body),
            _ => None,
        })
        .flat_map(|res| res.unwrap().chunks).collect(); //TODO: right method probably: https://docs.rs/futures/0.1/futures/sink/trait.Sink.html#method.with_flat_map

    // TODO: remove returned chunks from chunk list...
    
    // TODO: for every of these chunks, get it from the database with the full path
    //       and send file contents to the node.

    println!("Available node chunks: {:?}", available_node_chunks);

//    event_loop.run(node_chunks).map_err(|e| CreateError::from(e))
    Ok(())
}
