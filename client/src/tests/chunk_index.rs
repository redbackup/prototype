use super::test_data;
use chunk_index::schema::*;


#[test]
#[ignore]
fn create_chunk_index() {
    test_data::_prepare_chunk_index("create_chunk_index");
}

#[test]
#[ignore]
fn add_folders() {
    let chunk_index = test_data::_prepare_chunk_index("add_folder");
    let folder1 = test_data::_prepare_folder(&chunk_index);

    let folder2 = NewFolder {
        name: String::from("bibio"),
        parent_folder: Some(folder1.id),
    };

    assert!(chunk_index.add_folder(folder2).is_ok());
}

#[test]
#[ignore]
fn add_file() {
    let chunk_index = test_data::_prepare_chunk_index("add_file");
    let folder = test_data::_prepare_folder(&chunk_index);
    test_data::_prepare_file(&chunk_index, &folder);
}

#[test]
#[ignore]
fn add_chunks() {
    let chunk_index = test_data::_prepare_chunk_index("add_chunk");
    let folder = test_data::_prepare_folder(&chunk_index);
    let file = test_data::_prepare_file(&chunk_index, &folder);
    let chunk1 = test_data::_prepare_chunk(&chunk_index, &file);

    let chunk2 = NewChunk {
        chunk_identifier: String::from("f6056ef7890a99494c34951817c2ed4fd3608a8488ef0ae6f2afac93ed76854e"),
        file: file.id,
        predecessor: Some(chunk1.id),
    };
    chunk_index.add_chunk(chunk2).expect("Chunk could not be added");
}


#[test]
#[ignore]
fn get_all_chunks() {
    let chunk_index = test_data::_prepare_chunk_index("get_all_chunks");
    let folder = test_data::_prepare_folder(&chunk_index);
    let file = test_data::_prepare_file(&chunk_index, &folder);
    let chunk1 = test_data::_prepare_chunk(&chunk_index, &file);

    let chunks = chunk_index.get_all_chunks().expect("Could not get all chunks");

    assert_eq!(chunks, vec!(chunk1));
}

#[test]
#[ignore]
fn get_full_chunk_path(){
    let chunk_index = test_data::_prepare_chunk_index("get_full_chunk_path");
    let folder = test_data::_prepare_folder(&chunk_index);
    let file = test_data::_prepare_file(&chunk_index, &folder);
    let chunk1 = test_data::_prepare_chunk(&chunk_index, &file);

    let path = chunk_index.get_full_chunk_path(chunk1.file).expect("Could not get full chunk paths");

    assert_eq!(path, vec!(folder.name,file.name));
}
