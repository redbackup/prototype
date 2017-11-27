use chrono::{DateTime, Utc};

use chunk_table::Chunk; //, ChunkTable};
use redbackup_protocol::message::*;

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
