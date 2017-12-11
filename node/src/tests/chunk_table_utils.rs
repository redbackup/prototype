use chunk_table::ChunkTable;
use chunk_table::Chunk;
use std::fs;

pub struct ChunkTableUtils {}

impl ChunkTableUtils {
    #[allow(unused_must_use)] // as we are not interested in the result of fs::remove_file
    pub fn chunk_table_for_test(test_name: &str) -> ChunkTable {
        let database_url = format!("{}/test-database-node-{}.db", env!("OUT_DIR"), test_name);
        println!("Database file: {}", &database_url);
        fs::remove_file(&database_url);
        ChunkTable::new(&database_url).unwrap()
    }
    pub fn insert_and_verify(chunk_table: &ChunkTable, chunk: Chunk) -> Chunk {
        let added_chunk = chunk_table.add_chunk(&chunk).expect(
            "Chunk could not be added",
        );
        assert_eq!(chunk, added_chunk);
        chunk
    }
}
