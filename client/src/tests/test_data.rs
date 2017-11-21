use std::fs;
use chrono::prelude::*;
use chunk_index::ChunkIndex;
use chunk_index::schema::*;

#[allow(unused_must_use)] // as we are not interested in the result of fs::remove_file
pub fn _prepare_chunk_index(test_name: &str) -> ChunkIndex {
    let creation_date = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
    let file_name = format!("{}/test-chunk_index-{}.db", env!("OUT_DIR"), test_name);
    println!("chunk_index file: {}", &file_name);

    fs::remove_file(&file_name);
    ChunkIndex::new(&file_name, creation_date).expect("Chunk index could not be created")
}

pub fn _prepare_folder(chunk_index: &ChunkIndex) -> Folder {
    let folder = NewFolder {
        name: String::from("aisatsana"),
        parent_folder: None,
    };

    chunk_index.add_folder(folder.clone()).expect("Folder 1 could not be added")
}

pub fn _prepare_file(chunk_index: &ChunkIndex, folder: &Folder) -> File {
    let file = NewFile {
        name: String::from("bibio"),
        last_change_date: NaiveDate::from_ymd(2016, 11, 28).and_hms_milli(7, 8, 9, 10),
        folder: folder.id,
    };
    chunk_index.add_file(file).expect("File could not be added")
}

pub fn _prepare_chunk(chunk_index: &ChunkIndex, file: &File) -> Chunk {
    let chunk = NewChunk {
        chunk_identifier: String::from("7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2"),
        file: file.id,
        predecessor: None,
    };
    chunk_index.add_chunk(chunk).expect("Chunk could not be added")
}
