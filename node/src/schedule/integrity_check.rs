use futures_cpupool::CpuPool;
use futures_cpupool::CpuFuture;

use redbackup_storage::Storage;
use chunk_table::ChunkTable;

use super::Task;

pub struct IntegrityCheckTask {
    pool: CpuPool,
    storage: Storage,
    chunk_table: ChunkTable,
}
impl IntegrityCheckTask {
    pub fn new(storage: Storage, chunk_table: ChunkTable) -> Self {
        let pool = CpuPool::new(1);
        IntegrityCheckTask {
            storage,
            pool,
            chunk_table,
        }
    }
}
impl Task for IntegrityCheckTask {
    fn exec(&self) -> CpuFuture<(), ()> {
        let chunk_table = self.chunk_table.clone();
        let storage = self.storage.clone();
        self.pool.spawn_fn(move || {
            let db_res = chunk_table.load_random_chunks(5);
            if !db_res.is_err() {
                for chunk in db_res.unwrap() {
                    let vres = storage.verify(&chunk.chunk_identifier);
                    if vres.is_err() {
                        error!("Corruption detected: {}", vres.unwrap_err())
                    } else {
                        debug!(
                            "Integrity check for chunk {} successful",
                            chunk.chunk_identifier
                        );
                    }
                }
            } else {
                error!(
                    "Failed to load sample from db for integrity check: {}",
                    db_res.unwrap_err()
                );
            }
            let res: Result<(), ()> = Ok(());
            res
        })
    }

    fn name(&self) -> &'static str {
        "integrity check"
    }
}
