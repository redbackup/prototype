use super::test_data;
use create::chunk_index_builder::ChunkIndexBuilder;

#[ignore]
#[test]
fn file_checksum() {
    let mut path = test_data::_prepare_fs_structure("chunk_index_builder_file_checksum");
    path.push("documents/redbackup.txt");

    let checksum = ChunkIndexBuilder::file_checksum(&path)
        .expect("file_checksum returned an Error");
    let expected_checksum = "7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2";
    assert_eq!(checksum, expected_checksum);
}

#[ignore]
#[test]
fn new() {
    let fnname = "chunk_index_builder_new";
    let chunk_index = test_data::_prepare_chunk_index(fnname);
    let path = test_data::_prepare_fs_structure(fnname);
    ChunkIndexBuilder::new(&chunk_index, &path)
        .expect("Could not create chunk index builder");
}

#[ignore]
#[test]
fn build() {
    let fnname = "chunk_index_builder_build";
    let chunk_index = test_data::_prepare_chunk_index(fnname);
    let path = test_data::_prepare_fs_structure(fnname);
    let builder = ChunkIndexBuilder::new(&chunk_index, &path)
        .expect("Could not create chunk index builder");

    builder.build().expect("Could not build chunk_index");

    let chunks = chunk_index.get_all_chunks().expect("Could not get all chunks");
    let mut chunk_identifiers: Vec<String> = chunks.iter().map(|c| c.chunk_identifier.clone()).collect();
    chunk_identifiers.sort();

    let expected_chunk_identifiers = vec!(
        "0596c5800313885c1a4886e2b45f6389bc573c9487d892f02119d7f1f0ddf579",
        "7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2",
    );

    assert_eq!(chunk_identifiers, expected_chunk_identifiers);
}
