use std::fs;
use std::path::PathBuf;

use futures_cpupool::CpuPool;

use redbackup_protocol::message::ChunkContentElement;
use redbackup_storage::Storage;

use service::NodeService;
use super::chunk_table_utils::ChunkTableUtils;

pub struct ServiceUtils {}

impl ServiceUtils {
    pub fn service_for_test(test_name: &str) -> NodeService {
        let chunk_table = ChunkTableUtils::chunk_table_for_test(test_name);
        let storage = Self::storage_for_test(test_name);
        let cpu_pool = CpuPool::new_num_cpus();
        NodeService::new(cpu_pool, chunk_table, storage)
    }

    pub fn insert_and_verify(service: &NodeService, element: ChunkContentElement) {
        let chunk_table = service.chunk_table.clone();
        service.storage.persist(&element.chunk_identifier, &element.chunk_content).unwrap();
        let new_chunk = element.into();
        assert_eq!(new_chunk, chunk_table.add_chunk(&new_chunk).unwrap());
    }

    #[allow(unused_must_use)] // as we are not interested in the result of fs::remove_dir_all
    pub fn storage_for_test(test_name: &str) -> Storage {
        let storage_url = format!("{}/test-storage-{}", env!("OUT_DIR"), test_name);
        fs::remove_dir_all(&storage_url);
        Storage::new(PathBuf::from(&storage_url).to_owned()).unwrap()
    }

}
