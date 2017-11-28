use futures_cpupool::CpuPool;
use futures_cpupool::CpuFuture;

use redbackup_storage::Storage;
use chunk_table::{ChunkTable, DatabaseError};

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
            info!("begin with integrity check");
            check_integrity(chunk_table, storage).map_err(|e| {
                error!("integrity check has failed with a problem: {}", e);
                ()
            })
        })
    }

    fn name(&self) -> &'static str {
        "integrity check"
    }
}

quick_error!{
    #[derive(Debug)]
    pub enum IntegrityCheckError {
        DatabaseError(err: DatabaseError) {
            from()
            display("DatabaseError: {}", err)
            cause(err)
        }

    }
}


fn check_integrity(chunk_table: ChunkTable, storage: Storage) -> Result<(), IntegrityCheckError> {
    let chunks = chunk_table.load_random_chunks(5)?;
    for chunk in chunks {
        let res = storage.verify(&chunk.chunk_identifier);
        if res.is_err() {
            error!("Corruption detected: {}", res.unwrap_err())
        } else {
            debug!(
                "Integrity check for chunk {} successful",
                chunk.chunk_identifier
            );
        }
    }
   Ok(())
}
