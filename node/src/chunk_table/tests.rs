use chrono::NaiveDate;
use std::fs;

use super::ChunkTable;
use super::chunk::Chunk;

#[allow(unused_must_use)] // as we are not interested in the result of fs::remove_file
fn _prepare_chunk_table(test_name: &str) -> ChunkTable {
    let database_url = format!("{}/test-database-node-{}.db", env!("OUT_DIR"), test_name);
    println!("Database file: {}", &database_url);

    fs::remove_file(&database_url);
    ChunkTable::new(&database_url).expect("Chunk table could not be created")
}

fn _prepare_one_chunk(chunk_table: &ChunkTable) -> Chunk {
    // Note that tests might depend on these concrete values!
    let identifier = String::from("7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2");
    let date = NaiveDate::from_ymd(2014, 11, 28).and_hms_milli(7, 8, 9, 10);
    let expected_chunk = Chunk {
        chunk_identifier: identifier.clone(),
        expiration_date: date,
        root_handle: true,
    };

    let added_chunk = chunk_table.add_chunk(&identifier, date, true).expect("Chunk could not be added");
    assert_eq!(expected_chunk, added_chunk);
    
    expected_chunk
}


#[test]
fn create_chunk_table() {
    _prepare_chunk_table("create_chunk_table");
}

#[test]
fn add_chunk() {
    let chunk_table = _prepare_chunk_table("add_chunk");
    _prepare_one_chunk(&chunk_table);
}

#[test]
fn remove_chunk() {
    let chunk_table = _prepare_chunk_table("remove_chunk");
    let expected_chunk = _prepare_one_chunk(&chunk_table);

    let removed = chunk_table.remove_chunk(&expected_chunk.chunk_identifier).expect("Could not remove chunk");
    assert_eq!(removed, 1);
}

#[test]
fn get_chunk() {
    let chunk_table = _prepare_chunk_table("get_chunk");
    let expected_chunk = _prepare_one_chunk(&chunk_table);

    let got_chunk = chunk_table.get_chunk(&expected_chunk.chunk_identifier).expect("Could not remove chunk");
    assert_eq!(expected_chunk, got_chunk);
}

#[test]
fn update_chunk() {
    let chunk_table = _prepare_chunk_table("update_chunk");
    let original_chunk = _prepare_one_chunk(&chunk_table);

    let date = NaiveDate::from_ymd(2015, 11, 28).and_hms_milli(7, 8, 9, 10);
    let expected_chunk = Chunk {
        chunk_identifier: original_chunk.chunk_identifier.clone(),
        expiration_date: date,
        root_handle: true,
    };

    let updated_chunk = chunk_table.update_chunk(&original_chunk.chunk_identifier, date, true).expect("Could not remove chunk");
    assert_eq!(expected_chunk, updated_chunk);
}

#[test]
fn update_chunk_older_date() {
    let chunk_table = _prepare_chunk_table("update_chunk_older_date");
    let original_chunk = _prepare_one_chunk(&chunk_table);

    let second_date = NaiveDate::from_ymd(1970, 1, 1).and_hms_milli(7, 8, 9, 10);

    let updated_chunk = chunk_table.update_chunk(&original_chunk.chunk_identifier, second_date, false).expect("Could not remove chunk");
    assert_eq!(original_chunk, updated_chunk);
}
