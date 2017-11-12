use super::schema::*;

use super::snapshot::Snapshot;

// See http://docs.diesel.rs/diesel/associations/index.html for documentation on associations
#[derive(Queryable,Insertable,Identifiable,Associations,PartialEq,Clone,Debug)]
#[table_name = "snapshotchunks"]
#[belongs_to(Snapshot, foreign_key="snapshot_uuid")]
#[primary_key(snapshot_uuid,file_name,chunk_identifier)]
pub struct Snapshotchunk {
    pub snapshot_uuid: String,
    pub file_name: String,
    pub chunk_identifier: String,
}
