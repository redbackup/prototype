use std::io;

use futures::future;
use futures::Future;
use futures_cpupool::CpuPool;
use tokio_service::Service;
use chrono::{DateTime, Utc};

use redbackup_protocol::{Message, MessageKind};
use chunk_table::{Chunk, ChunkTable};
use redbackup_protocol::message::{ChunkElement, GetChunkStates, InternalError, InvalidRequest,
                                  ReturnChunkStates, ReturnDesignation};

#[cfg(test)]
mod tests;

pub struct NodeService {
    pub cpu_pool: CpuPool,
    pub chunk_table: ChunkTable,
}

impl Service for NodeService {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = io::Error>>;

    fn call(&self, request: Message) -> Self::Future {
        match request.body {
            MessageKind::GetDesignation(_) => self.handle_designation(),
            MessageKind::GetChunkStates(body) => self.handle_get_chunks(body),
            _ => self.handle_unknown(),
        }
    }
}

impl NodeService {
    pub fn new(cpu_pool: CpuPool, chunk_table: ChunkTable) -> NodeService {
        NodeService {
            cpu_pool,
            chunk_table,
        }
    }

    fn handle_unknown(&self) -> Box<Future<Item = Message, Error = io::Error>> {
        Box::new(future::ok(
            InvalidRequest::new("Node cannot handle this message kind"),
        ))
    }

    fn handle_designation(&self) -> Box<Future<Item = Message, Error = io::Error>> {
        Box::new(future::ok(ReturnDesignation::new(true)))
    }

    fn handle_get_chunks(
        &self,
        body: GetChunkStates,
    ) -> Box<Future<Item = Message, Error = io::Error>> {
        
        let chunk_table = self.chunk_table.clone();
        Box::new(self.cpu_pool.spawn_fn(move || -> Result<_, io::Error> {
            let db_chunks = body.chunks.into_iter().map(Chunk::from).collect();
            let result = chunk_table.get_and_update_chunks(db_chunks);
            if let Ok(results) = result {
                Ok(ReturnChunkStates::new(
                    results.into_iter().map(Chunk::into).collect(),
                ))
            } else {
                let msg = format!("A DB issue has occured: {}", result.unwrap_err());
                Ok(InternalError::new(&msg))
            }
        }))
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

impl Into<ChunkElement> for Chunk {
    fn into(self) -> ChunkElement {
        ChunkElement {
            chunk_identifier: self.chunk_identifier,
            expiration_date: DateTime::from_utc(self.expiration_date, Utc),
            root_handle: self.root_handle,
        }
    }
}
