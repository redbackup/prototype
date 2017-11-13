extern crate redbackup_storage;

use redbackup_storage::Storage;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

fn _get_test_target_path(test_name: &str) -> PathBuf {
    let target = format!("./target/testdata/{}", test_name);
    PathBuf::from(&target).to_owned()
}

fn _read_data(filename: &str) -> Vec<u8> {
    let mut f = File::open(filename).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).expect("failed to read file to the end...");
    buf
}

#[allow(unused_must_use)] // as we are not interested in the result of fs::remove_dir_all
fn _setup_empty_storage(test_name: &str) -> Storage {
    let target = _get_test_target_path(&test_name);
    std::fs::remove_dir_all(&target);
    let storage = Storage::new(target.clone()).unwrap();
    assert_eq!(target, storage.location());
    assert!(target.exists());
    assert!(target.is_dir());
    assert_eq!(std::fs::read_dir(&target).unwrap().count(), 0);
    storage
}

#[test]
fn create_storage_for_nonexisting_location() {
    _setup_empty_storage("create_storage_for_nonexisting_location");
}
#[test]
fn create_storage_for_existing_location() {
    let storage = _setup_empty_storage("create_storage_for_existing_location");
    // put some fake data into the directory....
    let example_file = storage.location().join("foo-baaa");
    File::create(&example_file).unwrap();
    assert!(example_file.exists());

    Storage::new(_get_test_target_path(
        "create_storage_for_existing_location",
    )).expect("Failed to create storage for an existing destination");
    assert!(example_file.exists())
}

#[test]
fn persist_and_get_data() {
    let storage = _setup_empty_storage("persist_and_get_data");
    let expected_data = _read_data("tests/data/lorem.txt");
    let identifier = "c1fcd4dd4dc0ee9208d7b9c6608b91bde8eee91b09bc5b4928b9371d5bdab16d";
    storage.persist(identifier, &expected_data).unwrap();
    let loaded_data = storage.get(identifier).unwrap();
    assert_eq!(loaded_data, expected_data);
    storage.delete(identifier).unwrap();
}

#[test]
fn ensure_persisting_existing_chunk_fails() {
    let storage = _setup_empty_storage("ensure_persisting_existing_chunk_fails");
    let expected_data = _read_data("tests/data/lorem.txt");
    let identifier = "c1fcd4dd4dc0ee9208d7b9c6608b91bde8eee91b09bc5b4928b9371d5bdab16d";
    storage.persist(identifier, &expected_data).unwrap();
    let err = storage.persist(identifier, &expected_data).unwrap_err();
    assert_eq!(format!("{}", err), "Can not persist already existing chunk with identifier c1fcd4dd4dc0ee9208d7b9c6608b91bde8eee91b09bc5b4928b9371d5bdab16d");
}
#[test]
fn ensure_deleting_nonexisting_chunk_fails() {
    let storage = _setup_empty_storage("ensure_deleting_nonexisting_chunk_fails");
    let identifier = "c1fcd4dd4dc0ee9208d7b9c6608b91bde8eee91b09bc5b4928b9371d5bdab16d";
    let err = storage.delete(identifier).unwrap_err();
    assert_eq!(format!("{}", err), "Can not delete non-existing chunk with identifier c1fcd4dd4dc0ee9208d7b9c6608b91bde8eee91b09bc5b4928b9371d5bdab16d");
}

#[test]
fn ensure_get_nonexisting_chunk_fails() {
    let storage = _setup_empty_storage("ensure_get_nonexisting_chunk_fails");
    let identifier = "c1fcd4dd4dc0ee9208d7b9c6608b91bde8eee91b09bc5b4928b9371d5bdab16d";
    let err = storage.get(identifier).unwrap_err();
    assert_eq!(format!("{}", err), "The chunk with the identifier c1fcd4dd4dc0ee9208d7b9c6608b91bde8eee91b09bc5b4928b9371d5bdab16d is not persisted");
}
