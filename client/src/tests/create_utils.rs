use create_backup::create_utils;
use super::test_data;

#[ignore]
#[test]
fn read_file_content() {
    let mut path = test_data::prepare_fs_structure("create_utils_file_hash");
    path.push("documents/redbackup.txt");
    let expected_content = "redbackup".to_string().into_bytes();

    let content = create_utils::read_file_content(&path)
        .expect("read_file_content returned an Error");
    assert_eq!(content, expected_content);
}

#[ignore]
#[test]
fn file_hash() {
    let mut path = test_data::prepare_fs_structure("create_utils_file_hash");
    path.push("documents/redbackup.txt");
    let expected_checksum = "7fcaddc8772aaa616f43361c217c23d308e933465b2099d00ba1418fec1839f2";


    let hash = create_utils::file_hash(&path)
        .expect("file_hash returned an Error");
    assert_eq!(hash, expected_checksum);
}
