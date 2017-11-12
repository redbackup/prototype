use super::schema::*;

#[derive(Queryable,Insertable,Identifiable,PartialEq,Clone,Debug)]
#[table_name = "chunks"]
#[primary_key(file_name,chunk_identifier)]
pub struct Chunk {
    pub file_name: String,
    pub chunk_identifier: String,
}
