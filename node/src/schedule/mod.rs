use std::net::SocketAddr;
use std::sync::Arc;
use std::time;
use std::time::Duration;

use futures::Future;
use futures_cpupool::CpuFuture;
use tokio_core::reactor::Handle;
use tokio_timer::Timer;


use redbackup_storage::Storage;
use chunk_table::ChunkTable;

mod integrity_check;
mod replication;

use self::integrity_check::IntegrityCheckTask;
use self::replication::ReplicateTask;


pub fn setup(
    handle: Handle,
    chunk_table: ChunkTable,
    storage: Storage,
    known_nodes: Vec<SocketAddr>,
) {
    info!("Setting up replication schedule..");
    let timeout = Duration::from_secs(30);
    let replication_task = ReplicateTask::new(storage.clone(), chunk_table.clone(), known_nodes);
    Schedule::new(handle.clone(), Arc::new(replication_task), timeout).schedule();

    info!("Setting up integrity check schedule..");
    let timeout = time::Duration::from_secs(60);
    let integrity_check_task = IntegrityCheckTask::new(storage, chunk_table);
    Schedule::new(handle.clone(), Arc::new(integrity_check_task), timeout).schedule();
}


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
