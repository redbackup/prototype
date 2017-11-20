use chrono::NaiveDate;

use super::chunk_table_utils::ChunkTableUtils;
use super::test_data::ExampleChunk;

#[test]
fn add_chunk() {
    let chunk_table = ChunkTableUtils::chunk_table_for_test("add_chunk");
    ChunkTableUtils::insert_and_verify(&chunk_table, ExampleChunk::one());
}

#[test]
fn remove_chunk() {
    let chunk_table = ChunkTableUtils::chunk_table_for_test("remove_chunk");
    let expected = ChunkTableUtils::insert_and_verify(&chunk_table, ExampleChunk::one());

    let removed = chunk_table
        .remove_chunk(&expected.chunk_identifier)
        .unwrap();

    assert_eq!(removed, 1);

    assert!(chunk_table.get_chunk(&expected.chunk_identifier).is_err());
}

#[test]
fn get_chunk() {
    let chunk_table = ChunkTableUtils::chunk_table_for_test("get_chunk");
    let expected = ChunkTableUtils::insert_and_verify(&chunk_table, ExampleChunk::one());

    let got_chunk = chunk_table
        .get_chunk(&expected.chunk_identifier)
        .unwrap();
    assert_eq!(expected, got_chunk);
}

#[test]
fn update_chunk() {
    let chunk_table = ChunkTableUtils::chunk_table_for_test("update_chunk");
    ChunkTableUtils::insert_and_verify(&chunk_table, ExampleChunk::one());

    let expexted = {
        let mut chunk = ExampleChunk::one();
        chunk.expiration_date = NaiveDate::from_ymd(2015, 11, 28).and_hms_milli(7, 8, 9, 10);
        chunk
    };

    let updated = chunk_table
        .update_chunk(&expexted)
        .unwrap();
    assert_eq!(expexted, updated);
}

#[test]
fn update_chunk_older_date() {
    let chunk_table = ChunkTableUtils::chunk_table_for_test("update_chunk_older_date");
    let original = ChunkTableUtils::insert_and_verify(&chunk_table, ExampleChunk::one());

    let second = {
        let mut chunk = ExampleChunk::one();
        chunk.expiration_date = NaiveDate::from_ymd(1970, 1, 1).and_hms_milli(7, 8, 9, 10);
        chunk.root_handle = false;
        chunk
    };
    
    let updated = chunk_table
        .update_chunk(&second)
        .expect("Could not remove chunk");
    assert_eq!(original, updated);
}
