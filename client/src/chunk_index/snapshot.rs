use diesel;

use self::diesel::prelude::*;
use chrono::prelude::*;

use super::{ChunkIndex,DatabaseError};
use super::chunk::Chunk;
use super::snapshotchunk::Snapshotchunk;

use super::schema::chunks::dsl::*;
use super::schema::{chunks, snapshotchunks, snapshots};

#[derive(Queryable,Insertable,Identifiable,PartialEq,Debug,Hash,Eq)]
#[table_name = "snapshots"]
#[primary_key(uuid)]
pub struct Snapshot {
    pub uuid: String, // TODO: Uuid?
    // diesel currently not support DateTime<Utc> in SQLite, just NaiveDateTime.
    pub creation_date: NaiveDateTime,
    pub expiration_date: NaiveDateTime,
}

impl Snapshot {
    /// This function adds the chunk to the chunk table (if not yet in there) and creates the
    /// chunk-snapshot relation.
    pub fn add_chunk(&self, chunk_index: &ChunkIndex, chunk: &Chunk) -> Result<(),DatabaseError> {
        let conn = chunk_index.db_pool.get()?;
        
        if let Err(_) = chunks.find((&chunk.file_name, &chunk.chunk_identifier)).first::<Chunk>(&*conn) {
            diesel::insert(chunk).into(chunks::table).execute(&*conn)?;
        }

        let snapshot_chunk = Snapshotchunk {
            snapshot_uuid: self.uuid.clone(),
            file_name: chunk.file_name.clone(),
            chunk_identifier: chunk.chunk_identifier.clone(),
        };
        diesel::insert(&snapshot_chunk).into(snapshotchunks::table).execute(&*conn)?;

        Ok(())
    }
}
