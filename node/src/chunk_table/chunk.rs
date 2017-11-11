use super::schema::*;
use chrono::prelude::*;

#[derive(Queryable,PartialEq,Debug)]
pub struct Chunk {
    pub chunk_identifier: String,
//    expiration_date: DateTime<Utc>,
    pub expiration_date: NaiveDateTime,
    pub root_handle: bool,
}

#[derive(Insertable)]
#[table_name = "chunks"]
pub struct NewChunk<'a> {
    pub chunk_identifier: &'a str,
//    expiration_date: DateTime<Utc>,
    pub expiration_date: NaiveDateTime,
    pub root_handle: bool,
}
