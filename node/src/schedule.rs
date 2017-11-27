use std::sync::Arc;
use std::time;
use std::time::Duration;

use futures::Future;
use futures_cpupool::CpuPool;
use futures_cpupool::CpuFuture;
use tokio_core::reactor::Handle;
use tokio_timer::Timer;

use redbackup_storage::Storage;
use chunk_table::ChunkTable;

trait Task {
    fn exec(&self) -> CpuFuture<(), ()>;
    fn name(&self) -> &'static str;
}

struct Schedule {
    task: Arc<Task>,
    handle: Handle,
    timeout: Duration,
    timer: Timer,
}

impl Schedule {
    fn new(handle: Handle, task: Arc<Task>, timeout: Duration) -> Self {
        let timer = Timer::default();
        Schedule {
            task,
            handle,
            timeout,
            timer,
        }
    }

    fn schedule(self) {
        let sleep = self.timer.sleep(self.timeout);
        let handle = self.handle.clone();
        let task = self.task.clone();

        handle.spawn(sleep.then(move |_| task.exec()).then(move |_| {
            debug!("rescheduling task {}...", self.task.name());
            self.schedule();
            Ok(())
        }));
    }
}

pub fn setup(handle: Handle, chunk_table: ChunkTable, storage: Storage) {
    // info!("Setting up replication schedule..");
    // let timeout = Duration::from_millis(5000);
    // let replication_task = ReplicateTask::new();
    // Schedule::new(handle.clone(), Arc::new(replication_task), timeout).schedule();

    info!("Setting up integrity check schedule..");
    let timeout = time::Duration::from_millis(20000);
    let integrity_check_task = IntegrityCheckTask::new(storage, chunk_table);
    Schedule::new(handle.clone(), Arc::new(integrity_check_task), timeout).schedule();
}

// struct ReplicateTask {
//     pool: CpuPool,
// }

// impl ReplicateTask {
//     fn new() -> Self {
//         let pool = CpuPool::new(1);
//         ReplicateTask { pool }
//     }
// }

// impl Task for ReplicateTask {
//     fn exec(&self) -> CpuFuture<(), ()> {
//         self.pool.spawn_fn(move || {
//             // TODO: Write actual logic...
//             let res: Result<(), ()> = Ok(());
//             res
//         })
//     }
//     fn name(&self) -> &'static str {
//         "replicate"
//     }
// }

struct IntegrityCheckTask {
    pool: CpuPool,
    storage: Storage,
    chunk_table: ChunkTable,
}
impl IntegrityCheckTask {
    fn new(storage: Storage, chunk_table: ChunkTable) -> Self {
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
