use super::schema::*;
use chrono::prelude::*;

use super::snapshot::Snapshot;

// See http://docs.diesel.rs/diesel/associations/index.html for documentation on associations
#[derive(Queryable,Insertable,Identifiable,Associations,PartialEq,Clone,Debug)]
#[table_name = "chunks"]
#[belongs_to(Snapshot, foreign_key="snapshot")]
#[primary_key(file,chunk_identifier)]
pub struct Chunk {
    pub file: String,
    pub chunk_identifier: String,
    pub expiration_date: NaiveDateTime,
    pub snapshot: String,
}
