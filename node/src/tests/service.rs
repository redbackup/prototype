use futures::Future;
use tokio_service::Service;
use chrono::Utc;

use redbackup_protocol::MessageKind;
use redbackup_protocol::message::{GetDesignation, ReturnDesignation, GetChunkStates, PostChunks, ChunkElement};

use chunk_table::Chunk;

use super::test_data::{ExampleChunkContentElement, ExampleChunkElement};
use super::service_utils::ServiceUtils;

#[test]
fn allways_give_designation() {
    let service = ServiceUtils::service_for_test("allways_give_designation");
    let req_msg = GetDesignation::new(0, Utc::now());
    let res_msg = service.call(req_msg).wait().unwrap();
    if let MessageKind::ReturnDesignation(body) = res_msg.body {
        assert!(body.designation);
    } else {
        panic!("Expected ReturnDesignation message!");
    }
}

#[test]
fn service_responds_if_wrong_message() {
    let service = ServiceUtils::service_for_test("service_responds_if_wrong_message");
    let req_msg = ReturnDesignation::new(false);
    let res_msg = service.call(req_msg).wait().unwrap();

    if let MessageKind::InvalidRequest(body) = res_msg.body {
        assert_eq!(body.reason, "Node cannot handle this message kind")
    } else {
        panic!("Expected ReturnDesignation message!");
    }
}

#[test]
fn empty_set_of_chunk_results_in_empty_response() {
    let service = ServiceUtils::service_for_test("empty_set_of_chunk_results_in_empty_response");
    let req_msg = GetChunkStates::new(Vec::new());
    let res_msg = service.call(req_msg).wait().unwrap();

    if let MessageKind::ReturnChunkStates(body) = res_msg.body {
        assert_eq!(body.chunks.len(), 0)
    } else {
        panic!("Expected ReturnChunkStates message!");
    }
}

#[test]
fn ensure_existing_chunks_are_updated() {
    let service = ServiceUtils::service_for_test("ensure_existing_chunks_are_updated");
    ServiceUtils::insert_and_verify(&service, ExampleChunkElement::one());

    let req_msg = GetChunkStates::new(vec![
        {
            let mut chunk = ExampleChunkElement::one();
            chunk.root_handle = true;
            chunk
        },
        ExampleChunkElement::two(),
    ]);
    let res_msg = service.call(req_msg).wait().unwrap();

    if let MessageKind::ReturnChunkStates(body) = res_msg.body {
        assert_eq!(body.chunks.len(), 1);
        let mut expected = ExampleChunkElement::one();
        expected.root_handle = true;
        assert_eq!(body.chunks[0], expected);
    } else {
        panic!("Expected ReturnChunkStates message!");
    }
}

#[test]
fn empty_set_of_post_chunks_results_in_empty_response() {
    let service = ServiceUtils::service_for_test("empty_set_of_post_chunks_results_in_empty_response");
    let req_msg = PostChunks::new(Vec::new());
    let res_msg = service.call(req_msg).wait().unwrap();

    if let MessageKind::AcknowledgeChunks(body) = res_msg.body {
        assert_eq!(body.chunks.len(), 0)
    } else {
        panic!("Expected AcknowledgeChunks message!");
    }
}

#[test]
fn post_non_existing_chunks() {
    let service = ServiceUtils::service_for_test("post_non_existing_chunks");
    let req_msg = PostChunks::new(vec![ExampleChunkContentElement::one()]);
    let res_msg = service.call(req_msg).wait().unwrap();

    if let MessageKind::AcknowledgeChunks(mut body) = res_msg.body {
        assert_eq!(body.chunks.len(), 1);
        // Ensure the chunk in the response and on the database are as expected
        let expected: Chunk = ExampleChunkContentElement::one().into();
        let service_chunk: Chunk = body.chunks.remove(0).into();
        let db_chunk = service
            .chunk_table
            .get_chunk(&expected.chunk_identifier)
            .unwrap();
        assert_eq!(service_chunk, expected);
        assert_eq!(db_chunk, expected);

        // Ensure that the contents in the storage are as expected
        let actual = service.storage.get(&expected.chunk_identifier).unwrap();
        assert_eq!(actual, ExampleChunkContentElement::one().chunk_content);
    } else {
        panic!("Expected AcknowledgeChunks message!");
    }
}

#[test]
fn post_existing_chunks_in_db() {
    let service = ServiceUtils::service_for_test("post_existing_chunks_in_db");
    ServiceUtils::insert_and_verify(
        &service,
        ExampleChunkContentElement::one().into(),
    );

    let req_msg = PostChunks::new(vec![
        ExampleChunkContentElement::one(),
        ExampleChunkContentElement::two(),
    ]);
    let res_msg = service.call(req_msg).wait().unwrap();

    if let MessageKind::AcknowledgeChunks(body) = res_msg.body {
        assert_eq!(body.chunks.len(), 1);
        let expected : ChunkElement = ExampleChunkContentElement::two().into();
        assert_eq!(body.chunks[0], expected);
    } else {
        panic!("Expected AcknowledgeChunks message!");
    }
}