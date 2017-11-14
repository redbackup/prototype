use super::*;
use chrono::prelude::*;
use futures::Future;
use redbackup_protocol::message::GetDesignation;
use redbackup_protocol::message::GetChunkStates;
use redbackup_protocol::message::InvalidRequest;

#[test]
fn allways_give_designation() {
    let service = NodeService::new();
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
    let service = NodeService::new();
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
    let service = NodeService::new();
    let get_chunk_states = GetChunkStates::new(Vec::new());
    let future = service.call(get_chunk_states);
    
    let message = future.wait().unwrap();
    if let MessageKind::ReturnChunkStates(body) = message.body {
        assert_eq!(body.chunks.len(), 0)
    } else {
        panic!("Expected ReturnDesignation message!");
    }
}