use restore_backup::utils;
use create_backup;
use super::test_data;

#[test]
fn restore_file_content() {
    let mut path = test_data::prepare_fs_structure("utils_restore_file_content");
    path.push("documents/redbackup2.txt");
    let content = "redbackup".to_string().into_bytes();

    utils::restore_file_content(&content, &path)
        .expect("restore_file_content returned an Error");

    assert!(path.is_file());

    let real_content = create_backup::create_utils::read_file_content(&path)
        .expect("Could not read file content for verification");
    assert_eq!(real_content, content);
}

#[test]
fn create_folder() {
    let mut path = test_data::prepare_fs_structure("utils_create_folder");
    path.push("movies");

    utils::create_folder(&path)
        .expect("create_folder returned an Error");
    assert!(path.is_dir());
}
