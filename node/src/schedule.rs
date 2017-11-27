use std::sync::Arc;
use std::time;
use std::time::Duration;

use futures::Future;
use futures_cpupool::CpuPool;
use futures_cpupool::CpuFuture;
use tokio_core::reactor::Handle;
use tokio_timer::Timer;

use redbackup_storage::Storage;

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

pub fn setup(handle: Handle, storage: Storage) {
    info!("Setting up replication schedule..");
    let timeout = Duration::from_millis(5000);
    let replication_task = ReplicateTask::new();
    Schedule::new(handle.clone(), Arc::new(replication_task), timeout).schedule();

    info!("Setting up integrity check schedule..");
    let timeout = time::Duration::from_millis(2000);
    let integrity_check_task = IntegrityCheckTask::new(storage);
    Schedule::new(handle.clone(), Arc::new(integrity_check_task), timeout).schedule();
}

struct ReplicateTask {
    pool: CpuPool,
}

impl ReplicateTask {
    fn new() -> Self {
        let pool = CpuPool::new(1);
        ReplicateTask { pool }
    }
}

impl Task for ReplicateTask {
    fn exec(&self) -> CpuFuture<(), ()> {
        self.pool.spawn_fn(move || {
            // TODO: Write actual logic...
            let res: Result<(), ()> = Ok(());
            res
        })
    }
    fn name(&self) -> &'static str {
        "replicate"
    }
}

struct IntegrityCheckTask {
    pool: CpuPool,
    storage: Storage,
}
impl IntegrityCheckTask {
    fn new(storage: Storage) -> Self {
        let pool = CpuPool::new(1);
        IntegrityCheckTask { storage, pool }
    }
}
impl Task for IntegrityCheckTask {
    fn exec(&self) -> CpuFuture<(), ()> {
        self.pool.spawn_fn(move || {
            // TODO: Write actual logic...
            let res: Result<(), ()> = Ok(());
            res
        })
    }
    fn name(&self) -> &'static str {
        "integrity check"
    }
}
