use std::error::Error;

use chrono::{DateTime, Utc};

use redbackup_storage::Storage;
use redbackup_protocol::message::*;

use chunk_table::Chunk;

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

/// Convert a Chunk to a ChunkContent Element.
/// This is no `From` or `Into` implementation, as it requires additional informatormation from
/// the storage and may fail.
pub fn chunk_to_chunk_contents_element(
    chunk: Chunk,
    storage: &Storage,
) -> Option<ChunkContentElement> {
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
