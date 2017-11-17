use super::*;
use futures::Future;
use redbackup_protocol::message::GetDesignation;
use redbackup_protocol::message::GetChunkStates;

impl ChunkTable {
    fn for_test(test_name: &str) -> ChunkTable {
        let database_url = format!("{}/test-database-node-{}.db", env!("OUT_DIR"), test_name);
        ChunkTable::new(&database_url).unwrap()
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

#[test]
fn allways_give_designation() {
    let service = NodeService::from_test_name("allways_give_designation");
    let get_designation = GetDesignation::new(0, Utc::now());
    let future = service.call(get_designation);
    let message = future.wait().unwrap();
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
    let future = service.call(get_chunk_states);
    
    let message = future.wait().unwrap();
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
    let future = service.call(get_chunk_states);
    
    let message = future.wait().unwrap();
    if let MessageKind::ReturnChunkStates(body) = message.body {
        assert_eq!(body.chunks.len(), 0)
    } else {
        panic!("Expected ReturnChunkStates message!");
    }
}
