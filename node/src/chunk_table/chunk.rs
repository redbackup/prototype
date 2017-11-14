use super::schema::*;
use chrono::prelude::*;

#[derive(Queryable,Insertable,Identifiable,PartialEq,Debug)]
#[table_name = "chunks"]
#[primary_key(chunk_identifier)]
pub struct Chunk {
    pub chunk_identifier: String,
    // diesel currently not support DateTime<Utc> in SQLite, just NaiveDateTime.
    pub expiration_date: NaiveDateTime,
    pub root_handle: bool,
}
