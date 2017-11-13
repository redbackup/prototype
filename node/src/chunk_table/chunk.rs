use super::schema::*;
use chrono::prelude::*;

#[derive(Queryable,Insertable,Identifiable,PartialEq,Debug)]
#[table_name = "chunks"]
#[primary_key(chunk_identifier)]
pub struct Chunk {
    pub chunk_identifier: String,
    pub expiration_date: NaiveDateTime,
    pub root_handle: bool,
}
