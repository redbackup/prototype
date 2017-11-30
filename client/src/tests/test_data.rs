use std::io::prelude::*;
use std::fs;
use std::path::PathBuf;

use chrono::prelude::*;
use chunk_index::ChunkIndex;
use chunk_index::schema::*;

#[allow(unused_must_use)] // as we are not interested in the result of fs::remove_file
pub fn prepare_chunk_index(test_name: &str) -> ChunkIndex {
    let creation_date = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
    let file_name = PathBuf::from(format!("{}/test-chunk_index-{}.db", env!("OUT_DIR"), test_name));
    println!("chunk_index file: {:?}", &file_name);

    fs::remove_file(&file_name);
    ChunkIndex::new(file_name, creation_date).expect("Chunk index could not be created")
}

pub fn prepare_folder(chunk_index: &ChunkIndex) -> Folder {
    let folder = NewFolder {
        name: String::from("aisatsana"),
        parent_folder: None,
    };

    chunk_index.add_folder(folder.clone()).expect("Folder 1 could not be added")
}

pub fn prepare_file(chunk_index: &ChunkIndex, folder: &Folder) -> File {
    let file = NewFile {
        name: String::from("bibio"),
        last_change_date: NaiveDate::from_ymd(2016, 11, 28).and_hms_milli(7, 8, 9, 10),
        folder: folder.id,
    };
    chunk_index.add_file(file).expect("File could not be added")
}

pub fn prepare_chunk(chunk_index: &ChunkIndex, file: &File) -> Chunk {
    let chunk = NewChunk {
        chunk_identifier: String::from("7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2"),
        file: file.id,
        predecessor: None,
    };
    chunk_index.add_chunk(chunk).expect("Chunk could not be added")
}


/// Creates a testing file struture.
///
/// Tree                     Hash (if file)
/// .
/// ├── app
/// │   └── hello_world.rs   0596c5800313885c1a4886e2b45f6389bc573c9487d892f02119d7f1f0ddf579
/// └── documents
///     └── redbackup.txt    7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2
///
pub fn prepare_fs_structure(test_name: &str) -> PathBuf {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);

    let mut root = PathBuf::from(env!("OUT_DIR"));
    root.push(test_name);
    if root.exists(){
        println!("Removing existing test directory {:?}", root);
        fs::remove_dir_all(&root).unwrap();
    }
    builder.create(&root).expect("Could not create testroot");

    let mut documents = root.clone();
    documents.push("documents");
    builder.create(&documents).expect("Could not create documents dir");

    documents.push("redbackup.txt");
    let mut redbackup_file = fs::File::create(&documents)
        .expect("Could not create redbackup test file");
    redbackup_file.write_all(b"redbackup")
        .expect("Could not write to redbackup test file");

    let mut app = root.clone();
    app.push("app");
    builder.create(&app).expect("Could not create app dir");

    app.push("hello_world.rs");
    let mut hello_world_file = fs::File::create(&app)
        .expect("Could not create hello_world test file");
    hello_world_file.write_all(b"fn main() {\n    println!(\"Hello, world!\");\n}")
        .expect("Could not write to hello_world test file");

    root
}
