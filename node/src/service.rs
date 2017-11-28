use std::io;

use futures::future;
use futures::Future;
use futures_cpupool::CpuPool;
use tokio_service::Service;
use chrono::{DateTime, Utc};
use std::error::Error;

use redbackup_protocol::{Message, MessageKind};
use redbackup_storage::Storage;
use chunk_table::{Chunk, ChunkTable};
use redbackup_protocol::message::*;

pub struct NodeService {
    pub cpu_pool: CpuPool,
    pub chunk_table: ChunkTable,
    pub storage: Storage,
}

impl Service for NodeService {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = io::Error>>;

    fn call(&self, request: Message) -> Self::Future {
        trace!("Handle request message {:?}", request);
        match request.body {
            MessageKind::GetDesignation(_) => self.handle_designation(),
            MessageKind::GetChunkStates(body) => self.handle_get_chunk_states(body),
            MessageKind::PostChunks(body) => self.handle_post_chunks(body),
            MessageKind::GetRootHandles(_) => self.handle_return_root_handles(),
            MessageKind::GetChunks(body) => self.handle_get_chunks(body),
            _ => self.handle_unknown(),
        }
    }
}

impl NodeService {
    pub fn new(cpu_pool: CpuPool, chunk_table: ChunkTable, storage: Storage) -> NodeService {
        NodeService {
            cpu_pool,
            chunk_table,
            storage,
        }
    }

    fn handle_unknown(&self) -> Box<Future<Item = Message, Error = io::Error>> {
        error!("Received unknown message kind");
        Box::new(future::ok(
            InvalidRequest::new("Node cannot handle this message kind"),
        ))
    }

    fn handle_designation(&self) -> Box<Future<Item = Message, Error = io::Error>> {
        info!("Grant designation");
        Box::new(future::ok(ReturnDesignation::new(true)))
    }

    fn handle_get_chunk_states(
        &self,
        body: GetChunkStates,
    ) -> Box<Future<Item = Message, Error = io::Error>> {
        info!("Return chunk states");
        let chunk_table = self.chunk_table.clone();
        Box::new(self.cpu_pool.spawn_fn(move || -> Result<_, io::Error> {
            let db_chunks = body.chunks.into_iter().map(Chunk::from).collect();
            debug!("Request state of chunks {:?} from chunk table", db_chunks);
            let result = chunk_table.get_and_update_chunks(db_chunks);
            if let Ok(results) = result {
                info!("Send available chunks to client");
                debug!("Available chunks: {:?}", results);
                Ok(ReturnChunkStates::new(
                    results.into_iter().map(Chunk::into).collect(),
                ))
            } else {
                let msg = format!("A DB issue has occured: {}", result.unwrap_err());
                Ok(InternalError::new(&msg))
            }
        }))
    }

    fn handle_post_chunks(
        &self,
        body: PostChunks,
    ) -> Box<Future<Item = Message, Error = io::Error>> {
        info!("Store posted chunks");
        let chunk_table = self.chunk_table.clone();
        let storage = self.storage.clone();

        Box::new(self.cpu_pool.spawn_fn(move || -> Result<_, io::Error> {
            let mut results = Vec::new();

            for chunk_content in body.chunks {
                if let Err(err) = storage.persist(
                    &chunk_content.chunk_identifier,
                    &chunk_content.chunk_content,
                ) {
                    error!("Failed to persist new chunk: {:?}", err.description());
                    continue;
                }

                let chunk = Chunk::from(chunk_content);
                let result = chunk_table.add_chunk(&chunk);
                if let Ok(new_chunk) = result {
                    debug!("Successfully stored chunk {}", new_chunk.chunk_identifier);
                    results.push(new_chunk);
                } else {
                    error!("Failed to insert new chunk: {:?}", result.unwrap_err());
                }
            }

            Ok(AcknowledgeChunks::new(
                results.into_iter().map(Chunk::into).collect(),
            ))
        }))
    }

    fn handle_return_root_handles(&self) -> Box<Future<Item = Message, Error = io::Error>> {
        info!("Return root handles");
        let chunk_table = self.chunk_table.clone();
        let storage = self.storage.clone();

        Box::new(self.cpu_pool.spawn_fn(move || -> Result<_, io::Error> {
            debug!("Get root handles from chunk table");
            match chunk_table.get_root_handles() {
                Ok(chunks) => {
                    let chunks = chunks
                        .into_iter()
                        .map(|chunk| chunk_to_chunk_contents_element(chunk, &storage))
                        .filter(|result| result.is_some())
                        .map(|r| r.unwrap())
                        .collect();
                    Ok(ReturnRootHandles::new(chunks))
                }
                Err(err) => {
                    let msg = format!("A DB issue has occured: {}", err);
                    Ok(InternalError::new(&msg))
                }
            }
        }))
    }
    fn handle_get_chunks(&self, body: GetChunks) -> Box<Future<Item = Message, Error = io::Error>> {
        info!("Return chunks");
        let chunk_table = self.chunk_table.clone();
        let storage = self.storage.clone();

        Box::new(self.cpu_pool.spawn_fn(move || -> Result<_, io::Error> {
            let mut results = Vec::new();
            for chunk_identifier in body.chunk_identifiers {
                debug!("Get chunk {} from chunk table", chunk_identifier);
                if let Ok(chunk) = chunk_table.get_chunk(&chunk_identifier) {
                    if let Some(chunk_content_element) =
                        chunk_to_chunk_contents_element(chunk, &storage)
                    {
                        debug!("Successfully received chunk {} from storage", chunk_content_element.chunk_identifier);
                        results.push(chunk_content_element);
                    }
                } else {
                    warn!("Failed to load requested chunk {}", chunk_identifier);
                }
            }
            Ok(ReturnChunks::new(results))
        }))
    }
}

fn chunk_to_chunk_contents_element(chunk: Chunk, storage: &Storage) -> Option<ChunkContentElement> {
    match storage.get(&chunk.chunk_identifier) {
        Err(err) => {
            warn!("Failed to load chunk: {:?}", err.description());
            None
        }
        Ok(content) => Some(ChunkContentElement {
            chunk_identifier: chunk.chunk_identifier,
            expiration_date: DateTime::from_utc(chunk.expiration_date, Utc),
            root_handle: chunk.root_handle,
            chunk_content: content,
        }),
    }
}


impl From<ChunkElement> for Chunk {
    fn from(other: ChunkElement) -> Self {
        Chunk {
            chunk_identifier: other.chunk_identifier,
            expiration_date: other.expiration_date.naive_utc(),
            root_handle: other.root_handle,
        }
    }
}

impl From<ChunkContentElement> for Chunk {
    fn from(other: ChunkContentElement) -> Self {
        Chunk {
            chunk_identifier: other.chunk_identifier,
            expiration_date: other.expiration_date.naive_utc(),
            root_handle: other.root_handle,
        }
    }
}

impl Into<ChunkElement> for Chunk {
    fn into(self) -> ChunkElement {
        ChunkElement {
            chunk_identifier: self.chunk_identifier,
            expiration_date: DateTime::from_utc(self.expiration_date, Utc),
            root_handle: self.root_handle,
        }
    }
}
