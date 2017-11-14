use chrono::prelude::*;

infer_schema!("env:DATABASE_FILE");

#[derive(Queryable, Identifiable, Associations, PartialEq, Clone, Debug)]
#[primary_key(id)]
#[belongs_to(Folder, foreign_key="parent_folder")]
pub struct Folder {
    pub id: i32,
    pub name: String,
    pub parent_folder: Option<i32>,
}

#[derive(Insertable, PartialEq, Clone, Debug)]
#[table_name = "folders"]
pub struct NewFolder {
    pub name: String,
    pub parent_folder: Option<i32>
}


#[derive(Queryable, Identifiable, Associations, PartialEq, Clone, Debug)]
#[table_name = "files"]
#[primary_key(id)]
#[belongs_to(Folder, foreign_key="folder")]
pub struct File {
    pub id: i32,
    pub name: String,
    pub last_change_date: NaiveDateTime,
    pub folder: i32,
}

#[derive(Insertable, PartialEq, Clone, Debug)]
#[table_name = "files"]
pub struct NewFile {
    pub name: String,
    pub last_change_date: NaiveDateTime,
    pub folder: i32,
}

#[derive(Queryable, Identifiable, Associations, PartialEq, Clone, Debug)]
#[primary_key(chunk_identifier,file)]
#[belongs_to(File, foreign_key="file")]
pub struct Chunk {
    pub id: i32,
    pub chunk_identifier: String,
    pub file: i32,
    pub predecessor: Option<i32>,
}

#[derive(Insertable, PartialEq, Clone, Debug)]
#[table_name = "chunks"]
pub struct NewChunk {
    pub chunk_identifier: String,
    pub file: i32,
    pub predecessor: Option<i32>,
}
