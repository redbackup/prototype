use glob::Pattern;

use super::test_data;
use create_backup::create_chunk_index::CreateChunkIndex;

#[test]
fn new() {
    let fnname = "create_chunk_index_new";
    let chunk_index = test_data::prepare_chunk_index(fnname);
    let path = test_data::prepare_fs_structure(fnname);
    CreateChunkIndex::new(&chunk_index, &path, &vec![]).expect(
        "Could not create chunk index builder",
    );
}

#[test]
fn build() {
    let fnname = "create_chunk_index_build";
    let chunk_index = test_data::prepare_chunk_index(fnname);
    let path = test_data::prepare_fs_structure(fnname);
    CreateChunkIndex::new(&chunk_index, &path, &vec![]).expect(
        "Could not create chunk index builder",
    );

    let chunks = chunk_index.get_all_chunks().expect(
        "Could not get all chunks",
    );
    let mut chunk_identifiers: Vec<String> =
        chunks.iter().map(|c| c.chunk_identifier.clone()).collect();
    chunk_identifiers.sort();

    let expected_chunk_identifiers = vec![
        "0596c5800313885c1a4886e2b45f6389bc573c9487d892f02119d7f1f0ddf579",
        "7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2",
    ];

    assert_eq!(chunk_identifiers, expected_chunk_identifiers);
}

#[test]
fn exclude_glob() {
    let fnname = "create_chunk_index_exclude_glob";
    let chunk_index = test_data::prepare_chunk_index(fnname);
    let path = test_data::prepare_fs_structure(fnname);
    let exclude = vec![Pattern::new("app/*.rs").unwrap()];
    CreateChunkIndex::new(&chunk_index, &path, &exclude)
        .expect("Could not create chunk index builder");

    let chunks = chunk_index.get_all_chunks().expect(
        "Could not get all chunks",
    );
    let mut chunk_identifiers: Vec<String> =
        chunks.iter().map(|c| c.chunk_identifier.clone()).collect();
    chunk_identifiers.sort();

    let expected_chunk_identifiers = vec![
        "7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2",
    ];

    assert_eq!(chunk_identifiers, expected_chunk_identifiers);
}
