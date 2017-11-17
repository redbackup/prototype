use super::*;
use futures::Future;
use chrono::{DateTime, NaiveDate, Utc};
use std::fs;

use redbackup_protocol::message::GetDesignation;
use redbackup_protocol::message::GetChunkStates;

impl ChunkTable {
    #[allow(unused_must_use)] // as we are not interested in the result of fs::remove_file
    fn for_test(test_name: &str) -> ChunkTable {
        let database_url = Self::db_url(test_name);
        fs::remove_file(&database_url);
        ChunkTable::new(&database_url).unwrap()
    }
    fn db_url(test_name: &str) -> String {
        format!("{}/test-database-node-{}.db", env!("OUT_DIR"), test_name)
    }
}

impl NodeService {
    fn from_test_name(test_name: &str) -> Self {
        let chunk_table = ChunkTable::for_test(test_name);
        Self::from_chunk_table(chunk_table)
    }
    fn from_chunk_table(chunk_table: ChunkTable) -> Self {
        let cpu_pool = CpuPool::new_num_cpus();
        Self::new(cpu_pool, chunk_table)
    }
}
fn chunk_element_1() -> ChunkElement {
    ChunkElement {
        chunk_identifier: String::from(
            "7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2",
        ),
        expiration_date: DateTime::from_utc(
            NaiveDate::from_ymd(2014, 11, 28).and_hms_milli(7, 8, 9, 10),
            Utc,
        ),
        root_handle: false,
    }
}
fn chunk_element_2() -> ChunkElement {
    ChunkElement {
        chunk_identifier: String::from(
            "41dde6e18fb531600e631311242bdab0983776cae9a9a7383d9bc6622f2732a5",
        ),
        expiration_date: DateTime::from_utc(
            NaiveDate::from_ymd(2019, 4, 10).and_hms_milli(9, 4, 5, 19),
            Utc,
        ),
        root_handle: false,
    }
}

#[test]
fn allways_give_designation() {
    let service = NodeService::from_test_name("allways_give_designation");
    let get_designation = GetDesignation::new(0, Utc::now());
    let message = service.call(get_designation).wait().unwrap();
    if let MessageKind::ReturnDesignation(body) = message.body {
        assert!(body.designation);
    } else {
        panic!("Expected ReturnDesignation message!");
    }
}

#[test]
fn service_responds_if_wrong_message() {
    let service = NodeService::from_test_name("service_responds_if_wrong_message");
    let get_chunk_states = ReturnDesignation::new(false);

    let message = service.call(get_chunk_states).wait().unwrap();

    if let MessageKind::InvalidRequest(body) = message.body {
        assert_eq!(body.reason, "Node cannot handle this message kind")
    } else {
        panic!("Expected ReturnDesignation message!");
    }
}

#[test]
fn empty_set_of_chunk_results_in_empty_response() {
    let service = NodeService::from_test_name("empty_set_of_chunk_results_in_empty_response");
    let get_chunk_states = GetChunkStates::new(Vec::new());

    let message = service.call(get_chunk_states).wait().unwrap();

    if let MessageKind::ReturnChunkStates(body) = message.body {
        assert_eq!(body.chunks.len(), 0)
    } else {
        panic!("Expected ReturnChunkStates message!");
    }
}

#[test]
fn ensure_existing_chunks_are_updated() {
    let chunk_table = ChunkTable::for_test("ensure_existing_chunks_are_updated");
    let existing_chunk: Chunk = chunk_element_1().into();
    assert_eq!(
        existing_chunk,
        chunk_table.add_chunk(&existing_chunk).unwrap()
    );

    let mut existing_element = chunk_element_1();
    existing_element.root_handle = true;
    let non_existing_element = chunk_element_2();


    let service = NodeService::from_chunk_table(chunk_table);
    let get_chunk_states = GetChunkStates::new(vec![existing_element, non_existing_element]);

    let message = service.call(get_chunk_states).wait().unwrap();

    if let MessageKind::ReturnChunkStates(body) = message.body {
        assert_eq!(body.chunks.len(), 1);
        let existing_chunk: ChunkElement = existing_chunk.into();
        assert_eq!(
            body.chunks[0].chunk_identifier,
            existing_chunk.chunk_identifier
        );
        assert_eq!(
            body.chunks[0].expiration_date,
            existing_chunk.expiration_date
        );
        assert!(body.chunks[0].root_handle);
    } else {
        panic!("Expected ReturnChunkStates message!");
    }
}
