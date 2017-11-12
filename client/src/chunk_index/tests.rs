use std::fs;
use chrono::NaiveDate;

use super::ChunkIndex;
use super::snapshot::Snapshot;
use super::chunk::Chunk;

#[allow(unused_must_use)] // as we are not interested in the result of fs::remove_file
fn _prepare_chunk_index(test_name: &str) -> ChunkIndex {
    let database_url = format!("{}/test-database-client-{}.db", env!("OUT_DIR"), test_name);
    println!("Database file: {}", &database_url);

    fs::remove_file(&database_url);
    ChunkIndex::new(&database_url).expect("Chunk index could not be created")
}

fn _prepare_snapshot(chunk_index: &ChunkIndex) -> Snapshot {
    let expected_snapshot = Snapshot{
        uuid: String::from("not yet generated"),
        creation_date: NaiveDate::from_ymd(2014, 11, 28).and_hms_milli(7, 8, 9, 10),
        expiration_date: NaiveDate::from_ymd(2016, 11, 28).and_hms_milli(7, 8, 9, 10),
    };

    let snapshot = chunk_index.add_snapshot(expected_snapshot.creation_date, expected_snapshot.expiration_date)
        .expect("Snapshot could not be added");
    assert_eq!(expected_snapshot.creation_date, snapshot.creation_date);

    snapshot
}
    


fn _prepare_one_chunk(chunk_index: &ChunkIndex, snapshot: &Snapshot) -> Chunk {
    // Note that tests might depend on these concrete values!
    let expected_chunk = Chunk {
        file_name: String::from("/tmp/redbackup-client/redbackup"),
        chunk_identifier: String::from("7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2"),
    };

    snapshot.add_chunk(&chunk_index, &expected_chunk).expect("Chunk could not be added");
    
    expected_chunk
}

#[test]
fn create_chunk_index() {
    _prepare_chunk_index("create_chunk_index");
}


#[test]
fn create_snapshot() {
    let chunk_index = _prepare_chunk_index("create_snapshot");
    _prepare_snapshot(&chunk_index);
}

#[test]
fn add_chunk() {
    let chunk_index = _prepare_chunk_index("add_chunk");
    let snapshot = _prepare_snapshot(&chunk_index);
    _prepare_one_chunk(&chunk_index, &snapshot);
}
